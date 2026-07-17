# ADR 0005: Render generated CV data directly through typed Leptos components

- **Status:** Accepted
- **Date:** 2026-07-17

## Context

Stages 1 and 2 produce one transactionally synchronized release containing
LaTeX, PDF, provenance metadata, and a validated `Cv<'static>` Rust value. The
website previously displayed a separately maintained copy of professional
content from `content.rs`, which could drift from that release.

Stage 3 needs responsive, accessible web presentation, safe imported inline
formatting, a matching PDF download, and meaningful tests without adding
runtime parsing, network requests, or another content schema.

## Alternatives

### Map generated data into existing hand-authored view models

This could reuse the old timeline component but would discard domain meaning,
require lossy mapping for typed dates and rich text, and preserve two parallel
models for the same content.

### Generate page-specific Leptos source

The importer could emit markup as well as data. That would couple
synchronization to visual design, make safe rendering harder to reason about,
and require regeneration for presentation-only changes.

### Render the generated domain model directly

A focused presentation module can format dates and locations, exhaustively map
safe inline variants to typed nodes, and compose responsive semantic sections
without copying content.

## Decision

Render `generated_cv::CV` directly through `cv_presentation::CvDocument` and
small components for repeated CV concepts. Keep the route thin. Match every
`Inline` variant to a Leptos node; do not create or inject HTML strings.

Use the generated profile for shared identity and professional links wherever
those values overlap. Restrict `content.rs` to non-CV editorial copy and the
separate portfolio catalogue. Serve the synchronized PDF from Trunk's static
`public/cv` copy and display the generated source tag and commit.

Preserve the complete Stage 2 model, including imported projects, but omit
projects from the CV route because the site has a dedicated Projects page. Use
the established two-column CV layout and timeline visual language from commit
`67cbe097` for experience and education.

Compile Leptos in CSR mode for Wasm and SSR mode for native targets. Native SSR
exists solely so repository-level integration tests can render the production
components to HTML; it does not introduce a deployed server.

## Rationale

- One imported value remains the source of truth from tag to visible page.
- Domain and parser boundaries remain independent of presentation.
- Exhaustive typed rendering preserves formatting without arbitrary HTML.
- Optional collections and artifact states are explicit and testable.
- Native rendering verifies actual semantic markup while production stays a
  static client-side application.

## Consequences

### Positive

- A merged synchronization update changes web content and PDF together.
- Manual CV drift is removed from the repository.
- The CV page avoids duplicating the dedicated Projects-page category.
- Presentation-only changes do not touch synchronization or parsing.
- Tests cover real component output, links, fallbacks, and provenance.

### Negative

- The CV presentation module knows the complete current domain shape and must
  handle any future domain variants exhaustively.
- Native and Wasm builds intentionally activate different Leptos render modes.
- Artifact availability cannot be probed reliably by a static client at build
  time, so the production pipeline remains responsible for publishing the
  synchronized bundle; the component's explicit unavailable state protects
  alternate composition and testing.
