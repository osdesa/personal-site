# Personal portfolio

A polished, responsive foundation for a long-term personal website and software
engineering portfolio. The application is built in Rust with Leptos, uses a
small typed content model, and deliberately contains clearly labelled sample
content until real CV and project information is supplied.

## Current status

The initial-design milestone is complete. The site includes:

- Home, Projects, CV and accessible not-found routes
- a fixed, non-scrolling homepage with minimal copy and no profile imagery
- responsive desktop, tablet and mobile layouts
- a permanent dark charcoal-and-grey visual system
- reusable navigation, project, skill, timeline and layout components
- central design tokens and typed portfolio data
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
├── .github/workflows/ci.yml    # Continuous integration
├── docs/                       # Architecture, design system and ADRs
├── public/                     # Static files copied into the bundle
├── src/
│   ├── app.rs                  # Router composition
│   ├── components/             # Reusable presentation components
│   ├── content.rs              # All editable portfolio data and models
│   ├── lib.rs                  # Shared library boundary for app and tests
│   ├── pages/                  # Route-level views
│   ├── routes.rs               # Canonical public route metadata
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

## Replacing placeholder information

All editable profile, social, project, skill, experience and education content
is in [`src/content.rs`](src/content.rs). Search that file for `Example`,
`Replace`, `placeholder`, `your-username` and `20XX` before
publishing. Page copy that explains the sample state can then be tightened in
`src/pages/`.

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

### Updating the CV PDF

Replace `public/cv/Hayden-Farrell-CV.pdf` while retaining the filename, then run
the production build and verify the download control. If the filename changes,
update `profile.cv_download_url` in `src/content.rs` at the same time.

## Continuous integration

`.github/workflows/ci.yml` runs for pull requests and pushes to `main`. It caches
Cargo and npm data, installs pinned frontend tooling, builds CSS, checks
formatting, denies Clippy warnings, runs all tests, performs a native release
build and creates the production WebAssembly bundle.

## Documentation

- [`docs/architecture.md`](docs/architecture.md) describes the implemented boundaries and data flow.
- [`docs/design-system.md`](docs/design-system.md) records tokens, responsive rules and component conventions.
- [`docs/adr/0001-initial-architecture.md`](docs/adr/0001-initial-architecture.md) records the initial architecture decision.

## Future work

The recommended next milestone is content replacement: add accurate personal
profile/CV information, real projects and the supplied CV PDF, then perform a
content and accessibility review. Later phases can add Markdown articles, RSS,
search, demonstrations and analytics only when their requirements are concrete.
