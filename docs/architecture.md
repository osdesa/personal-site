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
- **Domain data:** the immutable models and portfolio values in `content.rs`
- **Infrastructure boundary:** Trunk and the static `public/` directory

Content types do not depend on Leptos. Presentation reads the data and renders
it, while browser-specific behaviour is isolated from those data structures.

## Runtime flow

1. Trunk loads `index.html`, the generated stylesheet and the compiled Wasm.
2. The thin `main.rs` binary imports `App` from the library crate and mounts it
   into the document body.
3. `App` provides metadata context, starts `Router`, and wraps routes in
   `SiteShell`.
4. The active page consumes immutable values returned by `content::portfolio()`.

The home route is intentionally a fixed single-viewport composition. Its root
class hides the global footer and document overflow only while that route is
mounted. Interior routes retain normal document scrolling and the shared footer.

There are no network requests, environment variables, credentials or server
processes at runtime.

## Module responsibilities

### `src/content.rs`

Owns `Profile`, `SocialLink`, `Project`, `SkillGroup`, `TimelineItem` and
`Portfolio`. Static slices keep content updates obvious and type-safe. Pure
selection and integrity behaviour is exercised by `tests/content_tests.rs`.

### `src/lib.rs` and `src/main.rs`

`lib.rs` is the stable crate boundary shared by the browser executable and the
external tests. It publicly exposes application composition and the cohesive
content and route APIs that are useful outside their implementation
modules. `main.rs` only installs the panic hook and mounts `App`.

### `src/routes.rs`

Defines public navigation paths, labels and page-title mapping. Navigation and
tests share the same metadata; Leptos route declarations remain explicit in
`app.rs` so each view is visible at the application boundary.

### `src/components/`

Contains narrowly scoped reusable components:

- primitives: containers, headings, button links and skill badges
- site shell: header, responsive navigation, skip link and footer
- project cards and timeline entries

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
component rules. `styles/generated.css` is generated and ignored.

## Routing and static hosting

Leptos Router handles `/`, `/projects` and `/cv`, with a fallback view for
unknown paths. About and Contact are intentionally not separate routes. A
production static host must rewrite unknown
non-file paths to `index.html` so direct navigation can reach the client router.
That hosting rule belongs in future provider-specific deployment configuration.

## Testing strategy

All Rust tests—including unit-style tests—live in the repository-level `tests/`
directory and exercise the library crate's public behaviour. Source modules do
not contain inline test modules. The native suite covers stable logic rather
than browser markup snapshots:

- valid and unique project identifiers
- required content and HTTPS project links
- complete CV sections and absence of placeholder markers
- featured-project selection
- unique internal routes and page titles

CI additionally compiles every target with warnings denied and builds the actual
Wasm application. Visual and end-to-end browser automation can be added when
content stabilises and regression value exceeds maintenance cost.

## Extension points

- Replace static values in `content.rs` with parsed Markdown or JSON behind the
  same types when authoring needs justify it.
- Add project detail routes without changing the project-card content model.
- Add an Axum server only when a real server concern appears (for example a
  validated contact endpoint, authentication or dynamic content).
- Add article modules and generated route metadata when the blog milestone is
  approved.

No empty backend, CMS, repository layer or database adapter exists today.
