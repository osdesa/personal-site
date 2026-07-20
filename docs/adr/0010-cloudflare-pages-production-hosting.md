# ADR 0010: Cloudflare Pages production hosting

## Context

The static Leptos/Trunk application now has a live canonical domain and needs
repeatable hosting configuration without moving source synchronization or
credentials into the deployed runtime.

## Alternatives

- Keep hosting provider-neutral documentation.
- Add server-side rendering, Functions, or Workers for metadata and routing.
- Deploy the existing static output from Cloudflare Pages.

## Decision

Use Cloudflare Pages with Git integration. `main` is the production branch,
the repository root is the build root, and `dist/` is the published directory.
The canonical origin is `https://haydenfarrell.dev`; the `www` hostname redirects
to the apex. Pages uses `bash scripts/cloudflare-build.sh` with Node.js 24.

Keep the CSR application static: no Workers, Functions, runtime GitHub calls,
remote fonts, or deployment secrets. GitHub Actions remains the required CI
gate, and its synchronization secrets remain GitHub-only.

## Rationale

Pages matches the existing static artifact, supports automatic production
deployments from `main`, and provides deployment history for rollback. A
version-controlled build script avoids an opaque, fragile dashboard command.

## Consequences

- Generic initial metadata is intentionally site-wide; CSR cannot make it
  route-specific for non-rendering crawlers.
- Pages dashboard values and the `www` redirect remain operational settings.
- Static output validation detects missing release files, incorrect origins,
  preview/localhost leaks, and secret-name leakage before deployment.
