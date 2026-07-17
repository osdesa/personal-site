# ADR 0009: Reuse the production bundle for browser quality gates

## Context

The quality milestone requires formatting, Clippy with warnings denied, Rust
tests, a native release build, CSS generation, a production Trunk build,
browser accessibility checks and a meaningful performance budget. The previous
browser configuration launched `trunk serve --release`, which performed a
second release compilation after CI had already built the production bundle.
Lighthouse had been deferred after its independently launched Chrome process
was unreliable on the hosted runner.

## Alternatives

1. Keep browser and performance tools responsible for their own Trunk builds.
2. Run all quality checks in one sequential job and reuse its local `target/`
   and `dist/` directories.
3. Build `dist/` once, distribute it to dedicated browser-quality jobs, and run
   native and web compilation independently.

## Decision

Use independent `rust-quality` and `web-build` jobs for pull requests and
`main` commits. The Rust job runs formatting, Clippy, tests and
`cargo build --locked --release --all-features --bins`. The web job runs the
locked frontend install, CSS build and production Trunk build, then uploads
`dist/` as a one-day workflow artifact.

Browser accessibility checks download and serve that artifact through the
repository-owned static SPA server. The Lighthouse budget job does the same on
`main` only; it uses Playwright's installed Chromium path, CI-safe headless
flags and one retry for a failed browser launch. It retains the documented
three-run median budgets and does not retry a completed audit that exceeds a
budget.

Cargo build data, npm package downloads and the versioned Playwright browser
download are cached. Installed `node_modules` directories and workflow
artifacts are not used as cross-run dependency caches.

## Rationale

The artifact boundary gives browser and performance tools an auditable common
input: the exact release bundle produced by CI. It eliminates a redundant
Trunk compilation, permits native and Wasm work to overlap, and keeps the
expensive nine-navigation Lighthouse audit off the pull-request critical path
without weakening the main-branch release gate. The narrowly scoped launch
retry handles transient Chromium startup failures while preserving real budget
failures.

## Consequences

- Pull requests retain all required build and browser coverage with a shorter
  elapsed critical path.
- Every `main` commit additionally enforces Lighthouse's accessibility,
  performance, best-practices, SEO, transfer-size and layout-shift budgets.
- Artifact upload/download adds a small dependency between the web build and
  browser jobs, but `dist/` is small and this cost is lower than recompiling the
  Wasm bundle.
- CI must retain its cache keys when the lockfile changes; lockfile-keyed npm
  and Playwright caches invalidate automatically.
