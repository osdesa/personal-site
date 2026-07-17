# ADR 0008: Truthful CSR metadata and local Lighthouse budgets

## Context

The site is a static Trunk/Leptos client-side-rendered application without a
selected public host or canonical domain. Route pages can update browser
metadata after Wasm starts, but many social crawlers consume only the initial
HTML. The project also needed repeatable production-bundle performance checks
without adding runtime dependencies or a backend.

## Alternatives

- Treat client-side route metadata as route-specific social metadata.
- Add a guessed canonical domain, canonical URLs, sitemap and robots policy.
- Add static prerendering or server rendering now.
- Use only manual performance inspection.

## Decision

Keep route title and description definitions in `src/routes.rs`, with each
route module rendering them via `Title` and `Meta`. Keep initial HTML metadata
explicitly site-wide and omit all origin-dependent URLs until a domain is
chosen. Use generic static sharing metadata rather than adding a rendering
capability for route-specific previews. Emit only origin-independent JSON-LD
from public generated CV data.

Use current Lighthouse with a small repository-owned development runner. It
audits the built static SPA locally on three routes, records three desktop runs
and enforces median budgets for category scores, transferred bytes and
cumulative layout shift.

## Rationale

This gives browsers and JavaScript-capable crawlers specific metadata while
making no false promise to non-rendering social crawlers. It avoids a brittle
domain placeholder and makes the later hosting decision the single place where
absolute URL generation can be introduced. The Lighthouse runner measures the actual
release bundle and needs no production JavaScript, analytics service or server.

## Consequences

- Sharing previews remain site-wide textual fallbacks after the owner selects a
  domain; route-specific rendering is intentionally out of scope.
- `robots.txt`, `sitemap.xml`, canonical URLs and absolute social image URLs
  are deferred deployment work.
- CI gains a Chromium-based quality check and its local reports are ignored.
- Every new essential asset must fit the 550,000-byte initial-transfer budget
  or update the baseline and rationale deliberately.
