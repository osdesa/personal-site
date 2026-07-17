# Search, sharing and performance operations

## Current metadata boundary

The deployable application is a static, client-side-rendered Trunk bundle. Its
initial `index.html` is therefore the only dependable metadata source for
crawlers that do not execute WebAssembly. It contains a truthful, site-wide
title, description, Open Graph text fields and summary-card fields.

Each route also updates its title and description after the Leptos application
mounts. Those values are centralized in `src/routes.rs` and rendered by the
route page modules. They improve the browser experience and JavaScript-capable
crawlers, but do **not** provide route-specific social previews for
non-rendering crawlers.

No canonical URL, `og:url`, `og:image`, social-image URL, `robots.txt`, or
`sitemap.xml` is published yet. All need a selected public origin and stable
deployment routing. No social-card image is planned; the chosen sharing
strategy is the truthful generic text metadata in the initial document.

The mounted application also emits origin-independent `Person` and `WebSite`
JSON-LD from public generated CV identity and links. It excludes email and
does not claim a `url`. Like route metadata, it is not a guarantee for a
crawler that reads only initial HTML.

When a production domain is selected, add one canonical-origin configuration
value and use it to generate absolute canonical and Open Graph URLs. At the
same time, publish `robots.txt` and `sitemap.xml` containing only
`/`, `/projects` and `/cv`. Generic static sharing metadata is the selected
long-term approach; do not add prerendering or server rendering solely for
route-specific previews.

## Performance measurement and budgets

Build a production bundle, then run the Lighthouse audit against the generated
`dist/` directory:

```text
npm run css:build
trunk build --release
npm run test:performance
```

The repository-owned runner starts a local static SPA server, audits `/`,
`/projects` and `/cv` three times with the desktop preset, and evaluates median
results. The reports are written to the ignored `.lighthouseci/` directory
locally. On every `main` build, CI runs the same audit against the exact `dist/`
artifact produced by the production Trunk build. It explicitly selects the
cached Playwright Chromium executable, uses CI-safe headless flags, and retries
one failed browser launch before treating the check as an infrastructure failure.
Budget failures are never retried or averaged away. A deployed audit is still
required once hosting, HTTPS and cache headers exist.

The initial local release baseline on 17 July 2026 was a 0.32 s FCP and 0.68 s
LCP median across the nine route/runs; the initial transfer was 480,175 bytes
(about 469 KiB, principally 28 KiB CSS, 46 KiB JavaScript and 402 KiB Wasm).
The release build has no observed layout shift. These values justify the
following regression budgets rather than treating ideal local timings as hard
limits.

| Measure | CI threshold | Rationale |
| --- | --- | --- |
| Performance score | at least 0.90 | Allows normal local/CI variance while catching a material regression. |
| Accessibility, best-practices and SEO scores | 1.00 | Deterministic quality failures should be fixed, not averaged away. |
| Transferred bytes | at most 550,000 bytes | Leaves roughly 15% headroom over the baseline but rejects an additional large payload. |
| Cumulative layout shift | at most 0.05 | Leaves room for rendering variance while protecting stable card layout. |

The static release has no third-party scripts, remote fonts or runtime network
data. Project artwork is controlled local content with explicit dimensions,
asynchronous decoding and a reserved aspect ratio. If a project image fails or
is slow, the card's title and written project content remain usable. The CV
PDF is an optional download; a failed or slow request does not hide the full
web CV. Browser regression tests exercise both behaviours.

### Local Windows note

On Windows, Lighthouse can write a complete JSON report but fail while cleaning
up Chrome's temporary profile. The runner accepts that platform-specific exit
only when the report exists; malformed or missing reports still fail. A
deployed audit is still required after hosting adds HTTPS and cache headers.

## CI efficiency

Pull requests run the full Rust, CSS, production Trunk and browser-accessibility
coverage, but do not run the nine Lighthouse navigations. The performance gate
runs on `main`, where it protects the integration artifact without making review
feedback unnecessarily slow. The native quality job and web-build job start in
parallel. Browser and performance jobs consume the one-day `dist/` artifact, so
there is one CSS build and one Trunk build per workflow run rather than a second
release compilation for browser checks.

Cargo build data, npm's package-download cache and versioned Playwright Chromium
are cached. These caches are accelerators, not inputs to correctness: each Node
job still uses `npm ci`, and all browser checks run against the downloaded
artifact. See ADR 0009 for the complete job and cache contract.
