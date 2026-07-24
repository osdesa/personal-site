# Search, sharing and performance operations

## Production metadata boundary

The deployable application is a static, client-side-rendered Trunk bundle. Its
initial `index.html` is therefore the only dependable metadata source for
crawlers that do not execute WebAssembly. It contains a truthful, site-wide
title, description, Open Graph text fields and summary-card fields.

Each route also updates its title and description after the Leptos application
mounts. Those values are centralized in `src/routes.rs` and rendered by the
route page modules. They improve the browser experience and JavaScript-capable
crawlers, but do **not** provide route-specific social previews for
non-rendering crawlers.

The route metadata component removes the static description, canonical and Open
Graph URL fallback nodes before Leptos inserts the mounted route values. Browser
tests require exactly one of each node after mounting and require the Not Found
route to emit `noindex, nofollow`.

`src/routes.rs` owns the typed canonical production origin
`https://haydenfarrell.dev`. The generic initial document contains its home
canonical URL, Open Graph URL, title, description and controlled local sharing
image, plus the existing Twitter card fields. `public/robots.txt` permits normal
crawling and identifies the production sitemap. `public/sitemap.xml` contains
`/`, `/projects`, `/cv`, `/legal` and `/privacy`.

The mounted application also emits `Person` and `WebSite` JSON-LD from public
generated CV identity and links. It excludes email and identifies the canonical
site URL. Like route metadata, it is not a guarantee for a crawler that reads
only initial HTML.

The mounted application updates canonical and Open Graph URLs for browser and
JavaScript-capable crawlers. The static initial document stays generic for all
routes: a pure CSR bundle cannot make canonical metadata correct for every
direct route seen by non-rendering crawlers. Generic static sharing metadata is
the selected long-term approach; do not add prerendering or server rendering
solely for route-specific previews.

Validate the built directory with `npm run test:static` after a release build.
It rejects missing required static files, non-production origins, Pages preview
domains, localhost references, unresolved CSP placeholders, mismatched bootstrap
hashes, and the GitHub synchronization secret names.

## Static response security

`public/_headers` is copied into the bundle for Cloudflare Pages. It restricts
scripts, styles, images and connections to this origin, disables unnecessary
browser capabilities, prevents framing and MIME sniffing, and applies a strict
referrer policy. Trunk emits one inline Wasm bootstrap module, so a post-build
hook computes the SHA-256 of that exact generated script and substitutes it into
the CSP. The build fails if the number of inline modules or placeholders changes.

The local SPA server reads the finalized universal header block rather than
maintaining a second policy. Playwright and Lighthouse therefore exercise the
application under the same CSP as production. Fingerprinted top-level CSS,
JavaScript and Wasm files receive immutable one-year caching. Pages preview
hostnames receive `X-Robots-Tag: noindex`; the canonical hostname does not.

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
locally. On pull requests and every `main` build, CI runs the same audit against
the exact `dist/` artifact produced by the production Trunk build. It explicitly selects the
cached Playwright Chromium executable, uses CI-safe headless flags, and retries
one failed browser launch before treating the check as an infrastructure failure.
Budget failures are never retried or averaged away. A deployed audit is still
required once hosting, HTTPS and cache headers exist.

The production-readiness pass on 24 July 2026 recorded three desktop runs on
each core route before and after the changes. Both measurements used the
repository runner and local release bundle. Lighthouse was updated from 13.3.0
to 13.4.1 between measurements, so the SEO change must not be attributed only
to application code.

| Median across nine runs | Before | After |
| --- | ---: | ---: |
| Performance | 1.00 | 1.00 |
| Accessibility | 1.00 | 1.00 |
| Best Practices | 1.00 | 1.00 |
| SEO | 0.92 | 1.00 |
| First Contentful Paint | 322.44 ms | 322.55 ms |
| Largest Contentful Paint | 642.44 ms | 642.55 ms |
| Total Blocking Time | 0 ms | 0 ms |
| Cumulative Layout Shift | 0 | 0 |
| Speed Index | 322.44 ms | 322.55 ms |
| Transferred bytes | 464,894 | 466,312 |

The 1,418-byte transfer increase (0.31%) was measured after compatible Rust
dependency updates and remains well below the budget. Timing differences below
one millisecond are measurement noise. The release build has no observed layout
shift. These values justify the following regression budgets rather than
treating ideal local timings as hard limits.

| Measure | CI threshold | Rationale |
| --- | --- | --- |
| Performance score | at least 0.90 | Allows normal local/CI variance while catching a material regression. |
| Accessibility and best-practices scores | 1.00 | Deterministic quality failures should be fixed, not averaged away. |
| Local SEO score | at least 0.90 | Lighthouse treats client-route canonical links to the real HTTPS origin as cross-origin when it serves `dist/` from `127.0.0.1`; static-output validation checks those links exactly. Audit the deployed canonical origin at 1.00 before release. |
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

The native quality job and web-build job start in parallel. Browser and
performance jobs then consume the one-day `dist/` artifact, so there is one CSS
build and one Trunk build per workflow run rather than a second release
compilation for browser checks. Both browser jobs remain required on pull
requests so an accessibility or performance regression cannot merge.

Cargo build data, npm's package-download cache and versioned Playwright Chromium
are cached. These caches are accelerators, not inputs to correctness: each Node
job still uses `npm ci`, and all browser checks run against the downloaded
artifact. See ADR 0009 for the complete job and cache contract.
