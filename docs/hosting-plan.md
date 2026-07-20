# Hosting decision record and runbook

The hosting plan is complete: Cloudflare Pages deploys `main` from the
repository root to `dist/`, using Node.js 24, the Rust `wasm32-unknown-unknown`
target, and Trunk 0.21.14. The canonical origin is
`https://haydenfarrell.dev`; `www.haydenfarrell.dev` is secondary and should
redirect to the apex origin.

The required dashboard build command is `bash scripts/cloudflare-build.sh`.
Set it manually in Pages; the version-controlled script makes the exact build
reproducible but cannot modify Cloudflare settings. Keep the project free of
Workers, Functions, a top-level `404.html`, broad redirects, and GitHub sync
secrets. Pages must deploy automatically from `main`, while GitHub Actions CI
remains the required merge gate.

Validate direct public routes and assets after each configuration change using
the URLs in [`deployment.md`](deployment.md). Cloudflare's default SPA fallback
must serve non-file paths to the application, but assets such as the CV PDF
must remain real static responses. Roll back from the Pages deployment history
and revert application defects through a normal CI-protected pull request.
