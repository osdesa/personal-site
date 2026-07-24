# Production deployment

## Confirmed Cloudflare Pages configuration

Cloudflare Pages hosts the production static site from the `main` branch of
`osdesa/personal-site`. It builds at the repository root and publishes `dist/`.
The canonical HTTPS origin is `https://haydenfarrell.dev`; `www.haydenfarrell.dev`
is a secondary hostname and must permanently redirect to the apex domain while
preserving paths and query strings.

Pages deploys each merge to `main`. This delivery signal does not replace the
required GitHub Actions `CI` checks: Rust quality, dependency audits, production
bundle, accessibility and Lighthouse checks remain the correctness gate.
The scheduled CV and project workflows publish only through their trusted,
CI-gated automation PRs; see [`automation.md`](automation.md).

The Pages dashboard is intentionally free of `CV_SYNC_TOKEN`,
`PORTFOLIO_SYNC_TOKEN`, and `PORTFOLIO_GITHUB_TOKEN`. They are GitHub Actions
secrets only, are not Cloudflare environment variables, and never enter a
Trunk/client build. The deployed application has no runtime GitHub calls.

## Build command

The validated production command is:

```text
bash scripts/cloudflare-build.sh
```

The script preserves the confirmed build behaviour: it installs a minimal Rust
toolchain non-interactively, adds `wasm32-unknown-unknown`, installs Trunk
`0.21.14` with `--locked`, builds production CSS, and runs
`trunk build --release`. Cloudflare must also use Node.js 24. Updating the
Pages dashboard to this command is a manual setting; this repository cannot
change dashboard configuration automatically.

## Production verification

After a deployment, directly open and refresh:

- `https://haydenfarrell.dev/`
- `https://haydenfarrell.dev/projects`
- `https://haydenfarrell.dev/cv`
- `https://haydenfarrell.dev/legal`
- `https://haydenfarrell.dev/privacy`
- `https://haydenfarrell.dev/legal-notice` (permanent redirect to `/legal`)
- `https://haydenfarrell.dev/cv/Hayden-Farrell-CV.pdf`
- `https://haydenfarrell.dev/robots.txt`
- `https://haydenfarrell.dev/sitemap.xml`
- `https://haydenfarrell.dev/favicon.svg`
- `https://haydenfarrell.dev/images/project-default.svg`

Also test an unknown route. It should reach the Leptos Not Found view rather
than a host 404. Do not add a top-level `404.html` or broad catch-all redirect
unless a direct-route production test proves Pages needs it; both can interfere
with its normal SPA and asset handling.

Inspect response headers for HTML and fingerprinted `.css`, `.js` and `.wasm`
assets. Pages must apply the version-controlled `public/_headers` policy:
Content Security Policy, `Permissions-Policy`, `Referrer-Policy`,
`X-Content-Type-Options`, `X-Frame-Options`, immutable one-year asset caching,
and `X-Robots-Tag: noindex, nofollow` on Pages preview hostnames. Production
HTML remains indexable. Cloudflare provides compression and validators.

Inspect the initial HTML with JavaScript disabled: it must contain generic
site-wide canonical/Open Graph/Twitter metadata using the canonical origin.
Because the application is a static CSR SPA, that initial document cannot carry
a route-specific canonical URL for every direct client route. After Wasm
mounts, route metadata updates the browser view; non-rendering crawlers retain
the honest generic fallback. Server rendering is intentionally out of scope.

## Operations and rollback

For a failed deployment, inspect the relevant Cloudflare Pages deployment log.
To roll back, open **Workers & Pages** > this Pages project > **Deployments**,
select the last known-good production deployment, and use Cloudflare's rollback
or promote action. If the fault is source-related, also revert it through a
normal pull request so `main`, CI, and production converge.

To change the canonical domain in future, first configure and verify the new
HTTPS hostname and redirect policy in Pages/DNS. Then make one reviewed source
change to `PRODUCTION_ORIGIN` in `src/routes.rs`, update the static fallback
metadata in `index.html`, `public/robots.txt`, and `public/sitemap.xml`, update
these documents, run the static-output validation, and deploy from `main`.
