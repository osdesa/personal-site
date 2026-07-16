# ADR 0001: Initial portfolio architecture

- **Status:** Accepted
- **Date:** 2026-07-16

## Context

The first milestone needs a polished, responsive portfolio foundation with
typed sample content, reusable components, accessible routing, tests and CI. It
does not need authentication, a database, a CMS, form processing or other
server-owned behaviour.

The repository guide prefers Leptos, Rust, Tailwind CSS and a layered Cargo
architecture, while also requiring maintainability and simplicity ahead of
speculative structure.

## Alternatives

### Full-stack Leptos with Axum

This would support server rendering and future endpoints, but introduces Tokio,
server deployment, duplicated build modes and operational concerns that provide
no value for the current static content.

### Multi-crate Cargo workspace

Separate UI, shared, frontend and backend crates could enforce boundaries. At
this size those crates would mostly be empty wiring, increase dependency and CI
complexity, and make ordinary content changes harder to follow.

### Client-side Leptos in one organised crate

A single browser application can retain useful boundaries through Rust modules,
compile to static assets, and add crates later if an independently deployable or
reusable concern actually appears.

### Tailwind utilities only

Writing all styling as view-level utility strings would make tokens available,
but large Rust templates would become visually noisy and repeated component
states could drift.

### Plain CSS without Tailwind

This would minimise tooling, but would not meet the selected project stack and
would give up Tailwind's token/build pipeline for future component work.

## Decision

Use a single, client-side-rendered Leptos crate with Leptos Router and no backend.
Organise it into application composition, typed content, reusable components
and route pages. Provide a library crate boundary
for the browser executable and keep every Rust test in the root `tests/`
directory, including unit-style tests.

Use Tailwind CSS 4 as a local, pinned build dependency. Define primitive tokens
through `@theme`, semantic permanent-dark variables at the root, and cohesive
component classes in one source stylesheet. Trunk serves development builds and
produces the static WebAssembly release bundle.

Store portfolio content as immutable typed Rust values. Do not introduce a
database, Markdown parser or CMS until its authoring requirements are known.

## Rationale

- Static output matches the content-only runtime and can be hosted broadly.
- One crate keeps setup, tests and navigation understandable.
- Rust modules preserve clear responsibilities without empty architecture.
- Typed central data prevents content from scattering through components.
- Local Tailwind compilation avoids runtime CDN and JavaScript dependencies.
- Explicit semantic tokens keep palette and component changes consistent.
- Native tests validate pure logic quickly; Trunk validates the browser target.

## Consequences

### Positive

- Minimal runtime and dependency surface
- No server hosting or secret management for the initial site
- Straightforward content replacement and project addition
- Both native quality checks and real Wasm production builds in CI
- Backend or content-source choices remain open for later requirements
- No theme selector, persistence code or light-mode maintenance burden

### Negative

- Initial content is not available before the Wasm application loads.
- Static hosts need an `index.html` rewrite for direct client-route navigation.
- SEO beyond document metadata is weaker than pre-rendering or server rendering.
- Tailwind requires a Node build step in addition to Cargo/Trunk.

These costs are acceptable for the initial portfolio. If search indexing,
no-JavaScript rendering or dynamic content becomes a priority, a new ADR should
evaluate Leptos pre-rendering or Axum-based server rendering using evidence from
the deployed site.
