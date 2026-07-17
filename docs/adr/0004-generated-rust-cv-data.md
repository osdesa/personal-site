# ADR 0004: Generate static typed Rust data from CV LaTeX

- **Status:** Accepted
- **Date:** 2026-07-17

## Context

Stage 1 transactionally synchronizes a tagged LaTeX source and PDF from
`osdesa/cv`. The website needs a strongly typed, validated representation of
the same release before presentation work begins. The upstream document uses a
stable set of custom commands, so silently accepting broader LaTeX would hide
format drift and weaken the release boundary.

The representation must work in the existing static Leptos/Wasm application,
remain reviewable, avoid arbitrary HTML, and keep source, PDF, derived data and
provenance aligned.

## Alternatives

### Deserialize JSON in the browser

JSON is portable and easy to inspect. It introduces a parallel schema and a
runtime parse/failure path, allocates content in the browser, and only validates
compatibility when the application runs unless another build step is added.

### Generate a compact binary document

Binary data reduces an already insignificant file size. It is difficult to
review, still requires runtime decoding and schema versioning, and gives weaker
pull-request diagnostics.

### Parse LaTeX from `build.rs`

This avoids a checked-in derived file, but runs native import logic in every
build, complicates Wasm builds and dependency boundaries, and delays format
errors until compilation rather than the update transaction.

### Generate a checked-in Rust module

The importer can emit values of the shared Rust model using borrowed static
data. Normal compilation verifies all fields and variants, while the website
pays no runtime parsing or allocation cost.

## Decision

Implement a strict document-specific parser in the native `cv_sync` subsystem.
Require the expected custom-command declarations, consume the body completely
in its fixed section order, validate semantic values, and reject unsupported
commands with line/column diagnostics.

Represent content in the Leptos-independent `cv` domain module. Use typed dates,
locations and social platforms. Represent inline content only as text, strong,
emphasis, underline, and validated link nodes.

Generate `src/generated_cv.rs` deterministically as a `Cv<'static>` whose
strings and collections are borrowed through `Cow`. Include the source tag and
commit SHA as constants. Do not use JSON or a runtime decoder.

Upgrade the provenance manifest to schema version 2 and hash the generated
module alongside the LaTeX and PDF. Reparse and regenerate local data during
bundle validation. Install the generated module in the same rollback-capable
transaction, immediately before the manifest commit marker.

## Rationale

- Rust compilation provides schema compatibility and enum exhaustiveness.
- Borrowed static data has no runtime parser, allocations, or failure state.
- A generated Rust diff is reviewable with the source update.
- A narrow parser turns unexpected upstream format changes into explicit work.
- Re-generation checks establish semantic correspondence, not only file-hash
  consistency.
- The domain model remains independent of both importer and presentation.

## Consequences

### Positive

- Invalid or unsupported LaTeX cannot replace the deployed bundle.
- Source identity is visible in both generated code and the manifest.
- Stage 3 can consume one typed static value without transport concerns.
- Safe inline nodes prevent arbitrary HTML from crossing the import boundary.
- Deterministic regression tests expose parser or generator changes clearly.

### Negative

- Supported upstream grammar changes require parser, tests, and documentation
  updates.
- The repository stores a derived Rust file of roughly the same order of size
  as the source.
- Manifest schema 1 readers cannot consume the Stage 2 bundle.
- The current source's comma-separated skills and simple location convention
  constrain those fields until the upstream grammar is deliberately revised.
