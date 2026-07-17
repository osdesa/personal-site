# CV import pipeline

## Scope

Stage 2 converts the synchronized `osdesa/cv` LaTeX source into validated Rust
domain data. It is deliberately a document-specific importer, not a general
LaTeX implementation. A format change must be reviewed and added to this
contract before it can be published.

The pipeline does not render CV data. Stage 3 will consume the generated model.

## Supported document grammar

Comments beginning with an unescaped `%` and insignificant whitespace are
ignored. The non-content preamble is not interpreted, but it must declare all
of these custom commands:

- `\resumeItem`
- `\resumeSubheading`
- `\resumeSubSubheading`
- `\resumeProjectHeading`
- `\resumeSubItem`
- `\resumeSubHeadingListStart` and `\resumeSubHeadingListEnd`
- `\resumeItemListStart` and `\resumeItemListEnd`

The declarations must retain their current arities (one argument for items and
sub-items, four for subheadings, two for sub-subheadings and project headings,
and none for list boundaries). The body is consumed completely in this order:

```text
document          = heading education experience projects skills end-document
heading           = center(name, email-link, linkedin-link, github-link)
education         = section("Education"), subheading-list(education-entry+)
education-entry   = resumeSubheading(institution, location, qualification, dates)
experience        = section("Experience"), subheading-list(experience-entry+)
experience-entry  = resumeSubheading(role, dates, organisation, location), bullets
projects          = section("Projects"), subheading-list(project-entry+)
project-entry     = resumeProjectHeading(project-title "|" technologies, period), bullets
skills            = section("Technical Skills"), itemize(skill-group+)
skill-group       = textbf(category), { ":" comma-separated-values }
bullets           = resumeItemListStart, resumeItem(rich-text)+, resumeItemListEnd
```

The heading must retain the current `center`, `\textbf{\Huge \scshape ...}`,
line-break, and `\vspace{1pt}` structure. Its first link must be `mailto:` and
its label must equal the email address. Exactly one LinkedIn and one GitHub
HTTPS link are required. The Technical Skills section must retain the current
`itemize[leftmargin=0.15in, label={}]` and `\small{\item{...}}` wrapper.

Locations use `City, Country`; the final comma separates the fields. Date
ranges use `Month Year -- Month Year` or `Month Year -- Present`. Full and
conventional abbreviated English month names are accepted, with optional
abbreviation periods. Dates are validated and normalised to `CvDate`, `Month`,
`DateRange`, and `DateRangeEnd` values.

### Inline grammar

Rich-text fields support nested:

- plain UTF-8 text;
- `\textbf{...}` as `Inline::Strong`;
- `\emph{...}` and `\textit{...}` as `Inline::Emphasis`;
- `\underline{...}` as `Inline::Underline`;
- `\href{target}{label}` as `Inline::Link` when the target is absolute HTTPS
  or `mailto:`;
- escaped `\&`, `\%`, `\#`, `\_`, `\$`, `\{`, and `\}` characters.

Arbitrary HTML is neither accepted nor generated. Other commands, math mode,
unbraced groups, malformed nesting, non-HTTPS social links, empty required
values, unknown sections, and trailing body content are errors with one-based
line and column diagnostics.

## Parser architecture

`src/cv_sync/parser.rs` applies four stages:

1. replace comment bytes with spaces so source offsets remain stable;
2. verify the expected preamble command declarations;
3. use a bounded cursor and balanced-group reader to consume the exact body
   grammar and the recursive inline subset;
4. validate semantic constraints while constructing the owned `cv::Cv` model.

The domain model in `src/cv.rs` is independent of Leptos and the importer. It
uses `Cow` so parsing can return owned values while generated data can borrow
static strings and slices. It separates profile/contact/social data, education,
experience, projects, skills, locations, dates, and safe inline nodes.

`src/cv_sync/generator.rs` walks that model in source order and writes a
deterministic `src/generated_cv.rs`. The generated module exports `CV`,
`SOURCE_TAG`, and `SOURCE_COMMIT_SHA`.

## Output representation decision

The chosen representation is checked-in Rust source containing a
`Cv<'static>` built entirely from borrowed values.

Alternatives considered:

- **JSON:** simple and inspectable, but it duplicates a schema, moves validation
  to runtime or requires a build step, adds deserialization work to the browser,
  and does not make stale fields a compile error.
- **A compact binary format:** reduces bytes but is not reviewable, still needs
  runtime decoding and schema/version coordination, and offers no meaningful
  benefit for this small document.
- **Parsing LaTeX in `build.rs`:** avoids committing derived data, but every
  target build would compile/run native parsing dependencies, diagnostics would
  occur later, and source releases could not be reviewed as a complete prepared
  bundle.
- **Hand-maintained Rust content:** has strong types but can drift from the
  synchronized tag and defeats automation.

Generated Rust provides compile-time field and enum validation, no runtime
parser, no deserialization failure path, no content allocation, small build
complexity, and a readable pull-request diff. The manifest hash and regression
test prevent manual edits or generator drift.

## Transaction and failure behaviour

The synchronizer resolves the selected tag once and downloads both source
artifacts by its immutable commit SHA. It validates the PDF, strictly parses the
LaTeX, and generates Rust before entering the filesystem transaction.

The manifest schema is version 2. It records tag and commit provenance plus
repository-relative paths, byte lengths, and SHA-256 digests for the LaTeX,
PDF, and generated Rust module. On every run, the local store validates all
hashes, reparses the local LaTeX, regenerates the module using the manifest tag
and SHA, and requires an exact byte match before treating a tag as unchanged.

For an update, all four paths are staged and flushed on the repository
filesystem. Existing paths are moved to backups, then LaTeX, PDF, generated
Rust, and finally the manifest transaction marker are installed. A reported
replacement or finalisation failure removes installed candidates and restores
every backup. Network, download, parse, generation, validation, staging, and
lock errors happen before any current path is replaced.

The scheduled workflow includes all four paths in its fixed update pull
request. Reviewers therefore see one tag's source, downloadable PDF, generated
data, and metadata together.

## Stage 3 integration

Stage 3 should read `personal_site::generated_cv::CV` through the public types
in `personal_site::cv`. Presentation components should match `Inline` variants
to safe Leptos nodes and validate link behaviour; they must not convert rich
text to an HTML string or use raw HTML injection.

The current hand-authored `content.rs` remains the live presentation source
until Stage 3 explicitly maps or replaces those view models. Stage 2 does not
change website rendering.

## Assumptions and limitations

- Section names and order are fixed and each semantic section must be non-empty.
- The heading supports one email, LinkedIn, and GitHub only.
- Project technologies and skill values are comma-separated; commas cannot be
  embedded in one value.
- Locations split on the final comma and have no richer geographic semantics.
- Project periods remain rich text because the current source uses
  `Current Project` rather than a calendar range.
- Preamble layout definitions are not executed. Their expected declarations
  are checked, while all meaningful body content is parsed exhaustively.
