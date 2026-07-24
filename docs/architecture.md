# Architecture

## Overview

The portfolio is a client-side-rendered Leptos application compiled to
WebAssembly and distributed as static files. It has one Cargo package because
the initial site contains no server or independently reusable library that would
justify a workspace crate.

```text
index.html + generated CSS
          |
          v
     Leptos App/Router
          |
   +------+-------+
   |              |
site shell       pages
   |              |
   +----- reusable components
                  |
             typed content
```

This is a small layered architecture rather than an enterprise framework:

- **Presentation:** `pages/`, `components/` and `styles/input.css`
- **Application composition:** `lib.rs`, `app.rs`, `main.rs` and `routes.rs`
- **Domain data:** imported CV types/data in `cv.rs` and `generated_cv.rs`,
  generated GitHub project data in `projects.rs` and `generated_projects.rs`,
  plus homepage editorial values in `content.rs`
- **Infrastructure boundary:** Trunk and the static `public/` directory

The repository also contains a native build-time automation path. It is not
linked into the WebAssembly target:

```text
GitHub tag API ----> semantic selection ----> immutable commit SHA
                                                   |
GitHub raw files ---> PDF validation + strict LaTeX parser
                                                   |
                                                   v
                                  typed CV domain model
                                                   |
                                                   v
                                   static Rust generation
                                                   |
                                                   v
                              staged local transaction + manifest
                                                   |
                              +--------------------+------------------+
                              |                                       |
                              v                                       v
                  public/cv source bundle                  src/generated_cv.rs
```

Content types do not depend on Leptos. Presentation reads the data and renders
it, while browser-specific behaviour is isolated from those data structures.

The parallel native project path resolves the authenticated `portfolio` user
list through GraphQL, with topic and allowlist fallbacks, retrieves optional
`.github/portfolio.toml` metadata and `.github/thumbnail.png` artwork,
normalizes and filters complete candidates, sorts and caps them, and atomically
generates `src/generated_projects.rs` with matching local thumbnail assets. No
partial remote result reaches the checked-in artifact.

Scheduled GitHub Actions run the two native tools from trusted `main` code.
They update a fixed automation branch/PR only when bytes change, verify the
normal quality suite, and request GitHub native auto-merge only after
same-repository, fixed-branch, authenticated-author and workflow-marker checks.
Required CI and branch protection still decide when it may merge; Cloudflare
Pages then deploys the resulting `main` commit.

## Runtime flow

1. Trunk loads `index.html`, the generated stylesheet, controlled local image
   assets and the compiled Wasm. The initial document has truthful generic
   site-wide crawler metadata using the typed production origin from
   `routes.rs`; route-specific browser metadata updates after CSR mounting.
2. The thin `main.rs` binary imports `App` from the library crate and mounts it
   into the document body.
3. `App` provides metadata context, starts `Router`, and wraps routes in
   `SiteShell`.
4. Pages consume editorial values from `content::portfolio()`, CV identity from
   `generated_cv::CV`, and the shared static project slice from
   `generated_projects::PROJECTS`.

There are no runtime data requests, environment variables, credentials or
server processes in the deployed website runtime. Cloudflare Pages hosts the
`dist/` artifact built from `main`; it receives no synchronization credentials.
The separate native `sync-cv` and
`sync-projects` tools use GitHub only during local or scheduled maintenance.

## Module responsibilities

### `src/content.rs`

Owns homepage-specific editorial copy only. It deliberately does not contain
identity, contact, experience, education, skills, CV-project or portfolio
project data.

### `src/projects.rs`, `src/generated_projects.rs` and `src/project_sync/`

`projects.rs` defines the borrowed runtime project model.
`generated_projects.rs` is the automation-owned catalogue consumed by the home
and Projects pages. The native-only `project_sync` subsystem separates strict
configuration, GitHub transport, metadata parsing/normalization, deterministic
generation and atomic no-op-aware storage. Its `ProjectSource` boundary keeps
normal tests fixture-driven. Full operational rules live in
`docs/project-import.md`.

### `src/lib.rs` and `src/main.rs`

`lib.rs` is the stable crate boundary shared by the browser executable and the
external tests. It publicly exposes application composition and the cohesive
content and route APIs that are useful outside their implementation
modules. `main.rs` only installs the panic hook and mounts `App`.

### `src/routes.rs`

Defines public navigation paths, labels, titles and descriptions. Navigation,
browser metadata and tests share the same route metadata; Leptos route
declarations remain explicit in `app.rs` so each view is visible at the
application boundary. It also owns the typed `PRODUCTION_ORIGIN`, from which
route canonical URLs and controlled sharing artwork URLs are derived. The
static `index.html` is intentionally only a site-wide crawler/share fallback;
route-specific metadata appears after Wasm mounting and does not promise
route-specific social previews to non-rendering crawlers.

### `src/components/`

Contains narrowly scoped reusable site-wide components:

- primitives: containers, headings, button links and skill badges
- site shell: header, responsive inert navigation, skip link, main-focus
  restoration after mobile navigation and footer
- project cards
- origin-independent structured data generated from public CV identity

Components accept small typed values rather than broad configuration objects.
Links remain native anchors or router anchors.

The home page uses a restrained neutral-grey gradient rather than personal or
decorative imagery. It does not load a portrait, bitmap, or animation asset.

### `src/pages/`

Each route owns page composition and metadata. Pages do not define private
colour or spacing systems; they use shared component classes and design tokens.

### `styles/input.css`

Is the single styling source. Tailwind 4 supplies its build pipeline and token
utilities; semantic CSS variables implement the permanent dark palette and reusable
component rules. `styles/generated.css` is generated and ignored. Local project
imagery reserves its aspect ratio and uses explicit dimensions so failed or
slow artwork cannot move written card content.

### `src/cv.rs` and `src/generated_cv.rs`

`cv.rs` owns the presentation-independent imported CV domain: profile and
contact details, recognised social platforms, education, experience, projects,
skills, typed locations and month-precision date ranges. `RichText` is a safe
tree of text, strong, emphasis, underline and validated link nodes; it cannot
contain HTML.

The model uses `Cow`, allowing the native parser to construct owned values and
the generated module to expose borrowed static data without runtime parsing or
allocation. `generated_cv.rs` is an automation-owned artifact containing `CV`
and its upstream tag/SHA. `cv_presentation.rs` renders it directly; the CV route
adds no copied view model or hand-authored professional data.

### `src/cv_presentation.rs`

Owns Stage 3 presentation components for imported profile/contact data,
professional links, timeline entries, skill groups, PDF state, safe rich text,
provenance, and defensive unavailable states. Imported CV projects remain in
the Stage 2 domain model but are deliberately excluded from this route because
the site has a dedicated Projects page. The module exhaustively maps the closed
`Inline` tree to typed Leptos nodes, so imported text is escaped and raw HTML
cannot enter the view.

Wasm builds enable Leptos CSR. Native builds enable Leptos SSR only to render
the same components to HTML in external integration tests; the native binary
does not claim to provide a server. Runtime data remains a borrowed static Rust
value in both targets.

### `src/cv_sync/` and `src/bin/sync_cv.rs`

The native-only CV synchronization subsystem has six responsibilities:

- `manifest` owns semantic tag selection, schema-v2 provenance metadata,
  hashing, bounded TeX/PDF validation and candidate bundle construction;
- `github` adapts the paginated GitHub tag API and raw files to the small
  `CvSource` boundary;
- `parser` consumes the stable, documented CV grammar completely and builds the
  owned domain model with line/column diagnostics;
- `generator` deterministically writes the borrowed static Rust representation;
- `synchronizer` compares a fully validated local bundle with upstream and
  rejects moved tags or version rollback before downloading changes;
- `store` stages and flushes a candidate beside its destination, backs up the
  current LaTeX, PDF, generated module and manifest, installs the manifest last,
  and rolls back reported failures. Local validation reparses and regenerates
  the data to verify semantic correspondence in addition to manifest hashes.

The thin binary supplies the production GitHub source and repository-root
configuration. `cv_sync` and its native dependencies are excluded from the
`wasm32` target, so synchronization concerns cannot enter the browser bundle.
The domain model and generated static value do compile for Wasm, but no parser,
PDF, HTTP, hashing or serialization dependency does. The exact grammar and
Stage 3 contract are documented in `docs/cv-import.md`.

## Routing and static hosting

Leptos Router handles `/`, `/projects`, `/cv` and the footer-linked
`/legal-notice`, with a fallback view for unknown paths. About and Contact are
intentionally not separate routes.
Cloudflare Pages builds the repository root from `main` and serves `dist/` at
`https://haydenfarrell.dev`. Its normal SPA fallback must reach `index.html`
for non-file paths while serving assets unchanged. Do not add a top-level
`404.html` or broad catch-all redirect unless production direct-route testing
demonstrates that Pages requires one.

## Testing strategy

All Rust tests—including unit-style tests—live in the repository-level `tests/`
directory and exercise the library crate's public behaviour. Source modules do
not contain inline test modules. The native suite covers stable logic rather
than browser markup snapshots:

- valid and unique project identifiers
- required content and HTTPS project links
- generated profile and every major CV section in production Leptos markup
- conditional sections, PDF/data fallback states and download behaviour
- safe inline formatting, escaping, link policy and accessible link context
- date/location presentation and responsive/focus styling contracts
- featured-project selection
- unique internal routes and page titles
- semantic tag selection, manifest and artifact validation
- complete CV parsing, typed semantic values and nested safe formatting
- malformed structures, missing declarations/sections and unknown commands
- deterministic parsing/generation against the checked-in regression artifact
- GitHub transport behavior against a deterministic local HTTP server
- unchanged, update, corruption, unsupported-LaTeX, network, lock, tag-movement
  and rollback paths across all four committed paths
- integrity of the checked-in CV bundle against its manifest

CI additionally compiles every target with warnings denied and builds the actual
Wasm application. Playwright serves the built static SPA, scans the home,
projects, CV, legal-notice and not-found routes with axe, and checks the mobile menu's
keyboard, focus, overflow and reduced-motion behaviour. The scheduled CV
workflow repeats the native quality suite before opening an artifact update pull
request.

`scripts/validate-static-output.mjs` checks the release directory after Trunk
builds it. It verifies crawler files, controlled static assets, the CV PDF,
CSS/JavaScript/Wasm output, absolute metadata, the canonical route set, and
the absence of localhost, Pages-preview, and synchronization-token-name leaks.

The repository-owned Lighthouse runner separately serves `dist/` as a local SPA
after the release build, audits the three core portfolio routes three times, and
enforces median category, transfer-size and layout-shift budgets. It is a
development/CI dependency only; no Lighthouse, analytics or third-party script
enters the browser runtime.

## CI artifact and efficiency boundary

The CI workflow separates native Rust validation from the WebAssembly/CSS build
because the targets use different compilation outputs and neither needs to wait
for the other. The web-build job produces `dist/` once and uploads it as a
short-lived workflow artifact. Browser accessibility and the `main`-only
Lighthouse budget job each download that exact output and serve it through
`scripts/static-spa-server.mjs`; they do not run Trunk or Tailwind again.

Cargo caches may retain native and Wasm target directories because Cargo keeps
them in target-specific subdirectories. npm caches only downloaded packages,
and the Playwright cache contains only the versioned Chromium browser selected
by `package-lock.json`. `npm ci` remains mandatory in every Node job, so no
cached dependency tree is trusted as an installed workspace. A failed browser
or performance check therefore reports against the same release artifact that
passed the web-build job, while a fresh CI runner can still rebuild everything
from the lockfiles.

## Extension points

- Extend CV layout or component composition in `cv_presentation.rs` without
  changing the parser; change parsing only for an upstream grammar/domain change.
- Add project detail routes without changing the project-card content model.
- Add an Axum server only when a real server concern appears (for example a
  validated contact endpoint, authentication or dynamic content).
- Add article modules and generated route metadata when the blog milestone is
  approved.

No empty backend, CMS, repository layer or database adapter exists today.
