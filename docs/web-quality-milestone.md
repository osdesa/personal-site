# Web quality milestone

## Purpose

Strengthen the site as a public web experience before adding new product
features. The work must make accessibility, discoverability, performance and
production behaviour measurable and resistant to regression.

This is an engineering-quality milestone, not a redesign. Preserve the
existing permanent dark design, public routes (`/`, `/projects`, `/cv` and
`/legal-notice`),
typed generated CV/project content, and static Leptos/Trunk architecture.

## Current baseline

The site already has useful accessibility foundations:

- native landmarks, meaningful headings and a skip link;
- visible focus indicators and reduced-motion handling;
- labelled responsive navigation and explicit external-link context;
- a permanent dark token system;
- client-side Leptos routing, rendered as a static Trunk bundle.

Known gaps are assurance and public-web metadata rather than a lack of page
content. The project currently has Rust integration tests, but no browser-level
accessibility audit, no formal performance budget, no sitemap/robots files,
and no deployed host selected.

## Architecture constraints

- Keep browser runtime dependency-free apart from Leptos and existing assets.
- Do not add a backend, database, analytics, cookie banner, contact form or
  CMS as part of this milestone.
- Keep all colours, focus treatment and motion rules in `styles/input.css` and
  its design tokens. Do not introduce a light theme.
- Keep page metadata at the route/page boundary. Route paths and titles remain
  canonical in `src/routes.rs`.
- Keep Rust tests in `tests/`, as required by `AGENTS.md`. Browser checks may
  use a focused JavaScript test runner when that is the clearest boundary; do
  not duplicate the Rust unit/integration suite.
- Prefer native HTML and CSS over ARIA or JavaScript. Add ARIA only when native
  semantics cannot express the behaviour.
- Do not implement hosting configuration until a provider is selected.

## Workstreams and implementation order

### 1. Accessibility audit and regression checks

First establish repeatable evidence that the rendered application is usable.

1. Review every route at desktop, narrow mobile (320px) and short viewport
   height. Test keyboard-only navigation from the address bar through header,
   main content, controls and footer.
2. Audit focus visibility, contrast, heading order, landmark names, link
   purpose, image alternative text, touch target sizes, reduced motion and
   horizontal overflow.
3. Specifically validate the mobile navigation: correct `aria-expanded`
   state, closed links removed from the tab order, focus remains predictable,
   and menu selection returns the user to the document sensibly.
4. Add a small browser-level suite using a maintained accessibility scanner
   (for example, Playwright plus `@axe-core/playwright`) and focused behaviour
   assertions. Test the home, projects, CV and not-found views.
5. Fix every actionable violation at its semantic or component source, then
   add a targeted regression test where appropriate.

Do not treat an automated scan as a full audit: manual keyboard, zoom and
responsive checks remain required.

### 2. Search and sharing metadata

Make the public pages understandable to search engines and when shared.

1. Confirm one unique title and concise description for each route. Keep the
   `Title`/`Meta` components in route-level page modules.
2. Inspect the built `index.html` before promising sharing behaviour. The
   current client-side rendering can update browser metadata after Wasm loads,
   but social crawlers commonly read only the initial HTML. Choose and document
   one of: a truthful site-wide static preview in `index.html`, static
   prerendering, or a future server-rendered solution. Do not claim that
   client-side route metadata alone creates route-specific social previews.
3. Add a single production-site configuration value only after the canonical
   public origin is known. Use it to generate absolute canonical, Open Graph
   and social-card URLs; do not guess or hard-code a domain now.
4. Add an authored social preview image with an accessible, readable dark
   design. It should be a static asset, use stable dimensions, and be tested
   through generated metadata rather than manual page copy.
5. Add static `robots.txt` and `sitemap.xml` once the final public origin and
   deployment route behaviour are known. Include only canonical public pages.
6. Add valid `application/ld+json` structured data for `Person` and `WebSite`.
   Use values already published in generated CV data; never duplicate private
   contact data or invent credentials/claims.

Until a host and domain are selected, prepare the metadata structure and tests,
but leave absolute production URLs, sitemap and robots deployment values as
an explicitly documented follow-up.

### 3. Performance and resilience

Improve real-user loading and layout stability without speculative tuning.

1. Establish a production-build measurement process (Lighthouse or WebPageTest
   against a local production server first; deployed measurements later).
2. Record budgets in documentation and CI only after taking a baseline. At a
   minimum cover performance, accessibility, best practices, SEO, transferred
   bytes and layout shift. Budgets should be achievable and fail only on
   meaningful regressions.
3. Replace remote or unbounded image use with controlled assets where it
   materially improves reliability and size. Preserve meaningful image context
   and add explicit dimensions/aspect-ratio to prevent layout shift.
4. Verify the CSS and Wasm production bundle remain the only essential initial
   payload. Do not add third-party analytics, fonts or JavaScript libraries.
5. Test error and slow-network behaviour for project images and CV download.
   The core written content must remain readable and usable.

### 4. Deployment readiness

Implement this only when the owner chooses a host and public domain.

1. Follow `docs/deployment.md`: build with pinned dependencies, publish
   `dist/`, enable HTTPS and configure the SPA rewrite to `index.html`.
2. Verify direct navigation and refresh for `/`, `/projects`, `/cv`,
   `/legal-notice` and an
   unknown route on the deployed host.
3. Set the canonical origin, social image URLs, sitemap and robots policy.
4. Add preview deployment and deployment-status checks if supported by the
   chosen provider.
5. Run a final keyboard, mobile, Lighthouse and metadata validation on the
   public URL.

## Definition of done

The milestone is complete when:

- manual keyboard, responsive, zoom and reduced-motion checks pass for every
  route;
- automated accessibility checks cover the home, projects, CV and not-found
  states and run in CI;
- no actionable critical or serious accessibility issues remain;
- page titles/descriptions are unique and the planned canonical/social metadata
  is implemented without guessed production URLs;
- documented performance measurements and a justified budget exist;
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
  warnings`, `cargo test --all-targets --all-features`, `cargo build --release`,
  `npm run css:build`, and `trunk build --release` all pass;
- README, architecture/design-system docs and any ADR are updated when the
  implementation changes their documented contract.

## Decision points requiring owner input

- Which deployment provider and canonical domain to use.
- Whether an accessible professional headshot or a typography-only social card
  should represent the site when shared.
- Whether generic static sharing metadata is sufficient, or route-specific
  previews justify adding a prerendering/rendering capability later.
- Whether visitor analytics are wanted later. They are deliberately out of
  scope for this milestone and should be evaluated separately for privacy,
  consent and performance.

## Suggested commits

Keep commits narrow and Conventional:

1. `test: add browser accessibility regression checks`
2. `fix: resolve accessibility audit findings`
3. `feat: add route metadata and sharing assets`
4. `perf: establish production performance budgets`
5. `ci: enforce web quality checks`
6. `docs: document web quality operations`
