# ADR 0011: Static response and supply-chain security

## Context

The browser application is a static WebAssembly bundle with no runtime backend,
third-party scripts, remote fonts or form submission. Production still needs
explicit browser protections, appropriate caching, dependency auditing and a
reviewable policy for third-party CI code.

## Alternatives

- Rely on provider defaults for headers and dependency updates.
- Add a Cloudflare Worker only to attach response headers.
- Keep mutable major-version action tags for simpler workflow files.
- Store the complete policy in the repository and validate it with the bundle.

## Decision

Copy `public/_headers` into `dist/` and let Cloudflare Pages attach its Content
Security Policy, permissions restrictions, framing, referrer and MIME
protections. Fingerprinted CSS, JavaScript and WebAssembly assets receive
immutable one-year caching; Pages preview hosts receive a crawler `noindex`
header while the canonical hostname remains indexable.

Pin every GitHub Action to a full commit SHA with its release label retained as
a comment. Dependabot monitors Cargo, npm and GitHub Actions weekly. CI runs
`cargo audit`, `npm audit`, repository-owned script tests and the existing
compiler, browser and Lighthouse gates on pull requests.

## Rationale

The header file is versioned, reviewable and supported directly by the static
host, so a runtime component would add operational cost without adding
capability. Commit pinning prevents a moved tag from changing trusted CI code.
Automated update pull requests retain visibility and normal branch protection.

## Consequences

- Any future remote origin, form or runtime API requires a reviewed CSP change.
- The local SPA server mirrors the browser-enforced policy so Playwright and
  Lighthouse exercise the application under the production restrictions.
- Deployment verification must confirm that Pages attached the checked-in
  headers; static-output tests can verify the artifact but not provider state.
- Compatible dependency and action updates arrive as grouped pull requests;
  major upgrades remain separate and require deliberate review.
