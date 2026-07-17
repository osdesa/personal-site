# Personal portfolio

A polished, responsive personal website and software engineering portfolio.
The application is built in Rust with Leptos and uses a small typed content
model populated from Hayden Farrell's professional CV.

## Current status

The initial design and CV-content milestones are complete. The site includes:

- Home, Projects, CV and accessible not-found routes
- a fixed, non-scrolling homepage with minimal copy and no profile imagery
- responsive desktop, tablet and mobile layouts
- a permanent dark charcoal-and-grey visual system
- reusable navigation, project, skill, timeline and layout components
- central design tokens and typed portfolio data
- professional experience, education, and skills sourced from the current CV
- a transactional daily CV source/PDF importer with immutable provenance,
  strict LaTeX parsing and generated static Rust data
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
npm run css:build
trunk build --release
```

Tests intentionally target stable logic: content integrity, unique identifiers,
route metadata and content selection. Large generated HTML
snapshots are avoided because they would be brittle without improving confidence.
All Rust tests—including unit-style tests—live under `tests/`; source modules do
not contain inline `#[cfg(test)]` sections.

## Project structure

```text
.
├── .github/workflows/         # CI and scheduled CV synchronization
├── docs/                       # Architecture, design system and ADRs
├── public/                     # Static files copied into the bundle
├── src/
│   ├── app.rs                  # Router composition
│   ├── components/             # Reusable presentation components
│   ├── content.rs              # Current presentation content and models
│   ├── cv.rs                   # Imported CV domain model and safe rich text
│   ├── cv_sync/                # Native CV import/synchronization boundaries
│   ├── generated_cv.rs         # Transactionally generated static CV data
│   ├── lib.rs                  # Shared library boundary for app and tests
│   ├── pages/                  # Route-level views
│   ├── routes.rs               # Canonical public route metadata
│   ├── bin/sync_cv.rs          # Native synchronization CLI
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

Displayed profile, social, project, skill, experience and education content
remains in [`src/content.rs`](src/content.rs) through Stage 2. The automated CV
representation is available separately as `generated_cv::CV`; Stage 3 should
map or render that typed data rather than hand-copying future CV changes.
Project entries remain sample content until the later Projects-page milestone.
Route-level framing copy remains in `src/pages/`.

Update route-specific browser titles in
[`src/routes.rs`](src/routes.rs). Update the default HTML description and title
in `index.html` as a no-Wasm fallback.

### Adding a project

Add one `Project` value to the `PROJECTS` slice in `src/content.rs`:

- use a unique lowercase, hyphenated `id`
- provide a concise title and description
- add at least one technology
- provide an HTTPS repository URL
- set optional demo and image URLs to `None` when absent
- use `featured: true` only for projects that should receive featured metadata

The integrity tests catch duplicate/invalid identifiers and incomplete links.

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
byte for byte. See [`docs/cv-import.md`](docs/cv-import.md) for the supported
grammar, output decision and Stage 3 contract, and
[`public/cv/README.md`](public/cv/README.md) for operation details.

## Continuous integration

`.github/workflows/ci.yml` uses two event-specific paths:

- Open pull requests targeting `main` run the shorter Rust quality suite on
  creation, reopening, becoming ready for review and new commits: formatting,
  Clippy and tests. Feature-branch pushes without an open pull request do not
  start CI.
- Every commit pushed or merged into `main` runs the complete pipeline: pinned
  frontend tooling, CSS generation, formatting, Clippy, tests, a native release
  build and the production WebAssembly bundle.

`.github/workflows/sync-cv.yml` runs daily at 05:17 UTC and on manual dispatch.
It executes the release synchronizer, runs formatting, Clippy, tests and a
release tool build, then creates or updates `automation/cv-sync` only when the
validated bundle differs. It never pushes source artifacts directly to `main`.

Cargo and npm build data are cached where those tools are used. Pull request
runs supersede stale runs for the same pull request, while each `main` commit has
an independent full-build run.

## Documentation

- [`docs/architecture.md`](docs/architecture.md) describes the implemented boundaries and data flow.
- [`docs/cv-import.md`](docs/cv-import.md) specifies the supported LaTeX grammar, parser and Stage 3 contract.
- [`docs/design-system.md`](docs/design-system.md) records tokens, responsive rules and component conventions.
- [`docs/adr/0001-initial-architecture.md`](docs/adr/0001-initial-architecture.md) records the initial architecture decision.
- [`docs/adr/0002-event-specific-ci.md`](docs/adr/0002-event-specific-ci.md) records the event-specific CI strategy.
- [`docs/adr/0003-transactional-cv-synchronization.md`](docs/adr/0003-transactional-cv-synchronization.md) records the CV provenance and transaction design.
- [`docs/adr/0004-generated-rust-cv-data.md`](docs/adr/0004-generated-rust-cv-data.md) records the generated Rust representation decision.

## Future work

The recommended next CV milestone is Stage 3: render `generated_cv::CV` through
safe Leptos components without raw HTML. Browser-based visual/accessibility
validation and deployment configuration remain subsequent work. The Projects
page redesign, Markdown articles, RSS, search, demonstrations and analytics are
deferred until their requirements are concrete.
