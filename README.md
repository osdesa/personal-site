# Personal portfolio

A polished, responsive personal website and software engineering portfolio.
The application is built in Rust with Leptos. Its CV is generated from a
validated, versioned upstream release rather than maintained by hand.

## Current status

The initial design and automated CV presentation milestones are complete. The
site includes:

- Home, Projects, CV and accessible not-found routes
- a focused homepage with generated selected projects and no profile imagery
- responsive desktop, tablet and mobile layouts
- a permanent dark charcoal-and-grey visual system
- reusable navigation, project, CV-timeline, safe-rich-text and layout components
- central design tokens and typed portfolio data
- generated profile, contact, experience, education and skill content with
  visible source provenance
- a transactional daily CV source/PDF importer with immutable provenance,
  strict LaTeX parsing and generated static Rust data
- daily authenticated GitHub project synchronization with private-repository
  support, strict optional overrides and deterministic generated Rust data
- focused Rust tests and a production CI build

Authentication, a database, CMS, blog, analytics, search, contact-form backend
and deployment-provider configuration are intentionally outside this milestone.

## Technology stack

- stable Rust (edition 2024)
- [Leptos](https://leptos.dev/) in client-side-rendered mode
- [Leptos Router](https://docs.rs/leptos_router/) for browser routing
- [Tailwind CSS](https://tailwindcss.com/) 4 with local source CSS
- [Trunk](https://trunkrs.dev/) for the WebAssembly development server and bundle
- GitHub Actions for formatting, linting, tests and production builds

No backend is needed for the current content-focused site. See
[`docs/adr/0001-initial-architecture.md`](docs/adr/0001-initial-architecture.md)
for the decision and its consequences.

## Prerequisites

- the stable Rust toolchain (the dependencies require Rust 1.88 or newer)
- Node.js 24 and npm 11, matching CI
- Trunk 0.21.14

The checked-in `rust-toolchain.toml` installs the `rustfmt`, `clippy` and
`wasm32-unknown-unknown` components automatically through rustup.

Install Trunk once if it is not already available:

```text
cargo install trunk --version 0.21.14 --locked
```

## Local development

Install the pinned frontend dependencies:

```text
npm ci
```

Run these commands in separate terminals:

```text
npm run css:watch
trunk serve
```

Open `http://127.0.0.1:8080`. On Windows PowerShell, `npm.cmd` can be used in
place of `npm` if the local execution policy blocks `npm.ps1`.

## Production build

Generate minified CSS and the optimized WebAssembly bundle:

```text
npm run css:build
trunk build --release
npm run test:browser
```

The deployable static output is written to `dist/`. The native Cargo release
build is also kept healthy for fast compiler validation:

```text
cargo build --release
```

The native binary is not the deployable website; Trunk's WebAssembly bundle is.

## Quality checks

Run the same core checks used by CI:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
cargo build --release --features cv-sync --bin sync-cv
cargo build --release --features project-sync --bin sync-projects
npm run css:build
trunk build --release
```

Rust tests intentionally target stable logic: content integrity, unique identifiers,
route metadata and content selection. Browser tests serve the release-style Trunk
bundle, use axe to scan every public and not-found route, and cover the 320px
mobile menu, focus return, overflow and reduced-motion behaviour. Large generated HTML
snapshots are avoided because they would be brittle without improving confidence.
All Rust tests—including unit-style tests—live under `tests/`; source modules do
not contain inline `#[cfg(test)]` sections.

## Project structure

```text
.
├── .github/workflows/         # CI and scheduled content synchronization
├── docs/                       # Architecture, design system and ADRs
├── public/                     # Static files copied into the bundle
├── src/
│   ├── app.rs                  # Router composition
│   ├── components/             # Reusable presentation components
│   ├── content.rs              # Homepage-specific editorial content
│   ├── cv.rs                   # Imported CV domain model and safe rich text
│   ├── cv_presentation.rs      # Generated CV Leptos presentation
│   ├── cv_sync/                # Native CV import/synchronization boundaries
│   ├── generated_cv.rs         # Transactionally generated static CV data
│   ├── generated_projects.rs   # Atomically generated static project data
│   ├── project_sync/           # Native project synchronization boundaries
│   ├── projects.rs             # Portfolio project presentation domain
│   ├── lib.rs                  # Shared library boundary for app and tests
│   ├── pages/                  # Route-level views
│   ├── routes.rs               # Canonical public route metadata
│   ├── bin/                    # Native CV and project synchronization CLIs
│   └── main.rs                 # Browser application entry point
├── styles/input.css            # Tokens and component styling source
├── tests/                      # All Rust tests, including unit-style tests
├── index.html                  # Trunk HTML entry point
├── package.json                # Local Tailwind build
└── Trunk.toml                  # Development and bundle configuration
```

The single crate reflects the current application size. Modules preserve useful
boundaries without empty workspace crates or a speculative backend.
`src/lib.rs` provides the shared crate boundary for the thin browser binary and
the external test files in `tests/`, which are organised by responsibility.

## Updating portfolio content

[`src/generated_cv.rs`](src/generated_cv.rs) is the single source of truth for
displayed identity, contact, professional links, experience, education, CV
technical skills and source provenance. Imported CV projects remain in the
generated model for document fidelity but are intentionally not displayed on
the CV page because `/projects` is the dedicated project presentation. The
generated file is automation-owned; do not edit it or copy its values into page
components. [`src/content.rs`](src/content.rs) contains only non-CV homepage
copy. [`src/generated_projects.rs`](src/generated_projects.rs) is the single
source of truth for both project presentation surfaces and must not be edited
by hand. Route-level framing copy remains in `src/pages/`.

Extend CV presentation in
[`src/cv_presentation.rs`](src/cv_presentation.rs) by composing the existing
domain types. Add a component only when a semantic pattern repeats. Parser or
generator changes are required only when the upstream document grammar or
domain meaning changes, not for layout, copy framing, or styling changes.

Update route-specific browser titles in
[`src/routes.rs`](src/routes.rs). Update the default HTML description and title
in `index.html` as a no-Wasm fallback.

### Synchronizing portfolio projects

GitHub's GraphQL user-list API makes the named `portfolio` starred list the
primary source of truth. The `portfolio` repository topic and the seeded
allowlist in `portfolio-projects.toml` are ordered fallbacks. A daily
authenticated workflow reads public and private
repository metadata plus optional `.github/portfolio.toml` overrides, excludes
archives and forks by default, sorts by portfolio/creation date, keeps four,
and atomically generates `src/generated_projects.rs`.

The `PORTFOLIO_GITHUB_TOKEN` Actions secret needs fine-grained Starring,
Metadata and Contents read access. Run the synchronization manually with:

```text
cargo run --locked --release --features project-sync --bin sync-projects -- --root .
```

See [`docs/project-import.md`](docs/project-import.md) for selection, private
repository publication, metadata fields, token setup, ordering and fallback
rules.

### Synchronizing the CV source release

The canonical TeX and PDF are published as semantic-version tags in
[`osdesa/cv`](https://github.com/osdesa/cv). A daily GitHub Actions workflow
selects the highest version, resolves it to a commit SHA, validates both files,
strictly parses the supported LaTeX grammar, and generates
`src/generated_cv.rs`. It opens a pull request containing the source, PDF,
generated typed data and schema-v2 provenance manifest as one transaction.

Run the same operation locally with:

```text
cargo run --locked --release --features cv-sync --bin sync-cv -- --root .
```

The command is a no-op only when the manifest identifies the selected tag/SHA,
all hashes match, and reparsing the local TeX reproduces the generated module
byte for byte. When the synchronization pull request is merged, normal site
compilation renders the new `generated_cv::CV`, displays its tag and commit,
and Trunk serves the matching PDF from `/cv/Hayden-Farrell-CV.pdf`. No runtime
GitHub request, LaTeX parsing, or data decoding occurs. See
[`docs/cv-import.md`](docs/cv-import.md) for the supported grammar and complete
presentation contract, and
[`public/cv/README.md`](public/cv/README.md) for operation details.

## Continuous integration

`.github/workflows/ci.yml` uses two event-specific paths:

- Open pull requests targeting `main` run formatting, Clippy, Rust tests and the
  browser accessibility regression suite on creation, reopening, becoming ready
  for review and new commits. Feature-branch pushes without an open pull request
  do not start CI.
- Every commit pushed or merged into `main` runs the complete pipeline: pinned
  frontend tooling, CSS generation, formatting, Clippy, tests, a native release
  build and the production WebAssembly bundle.

`.github/workflows/sync-cv.yml` runs daily at 05:17 UTC and on manual dispatch.
It executes the release synchronizer, runs formatting, Clippy, tests and a
release tool build, then creates or updates `automation/cv-sync` only when the
validated bundle differs. It never pushes source artifacts directly to `main`.

`.github/workflows/sync-projects.yml` runs daily at 05:41 UTC and on manual
dispatch. It requires `PORTFOLIO_GITHUB_TOKEN`, preserves the current generated
catalogue on any failure, validates the repository, and opens or updates the
`automation/project-sync` pull request only when output differs.

Cargo and npm build data are cached where those tools are used. Pull request
runs supersede stale runs for the same pull request, while each `main` commit has
an independent full-build run.

## Documentation

- [`docs/architecture.md`](docs/architecture.md) describes the implemented boundaries and data flow.
- [`docs/cv-import.md`](docs/cv-import.md) specifies the supported LaTeX grammar, parser and Stage 3 presentation contract.
- [`docs/project-import.md`](docs/project-import.md) documents authenticated project selection, metadata and operation.
- [`docs/web-quality-milestone.md`](docs/web-quality-milestone.md) defines the staged accessibility, metadata, performance and deployment-readiness milestone.
- [`docs/design-system.md`](docs/design-system.md) records tokens, responsive rules and component conventions.
- [`docs/adr/0001-initial-architecture.md`](docs/adr/0001-initial-architecture.md) records the initial architecture decision.
- [`docs/adr/0002-event-specific-ci.md`](docs/adr/0002-event-specific-ci.md) records the event-specific CI strategy.
- [`docs/adr/0003-transactional-cv-synchronization.md`](docs/adr/0003-transactional-cv-synchronization.md) records the CV provenance and transaction design.
- [`docs/adr/0004-generated-rust-cv-data.md`](docs/adr/0004-generated-rust-cv-data.md) records the generated Rust representation decision.
- [`docs/adr/0005-generated-cv-presentation.md`](docs/adr/0005-generated-cv-presentation.md) records the direct generated-data presentation decision.
- [`docs/adr/0006-generated-github-projects.md`](docs/adr/0006-generated-github-projects.md) records build-time GitHub project generation.

## Future work

Provider-specific deployment configuration remains future work. Markdown
articles, RSS, search, richer demonstrations and analytics are
deferred until their requirements are concrete.
