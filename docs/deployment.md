# Deployment notes

## Current status

The website is intentionally not deployed yet. It builds as a static WebAssembly
application with Trunk, and the deployable output is the `dist/` directory.
No hosting provider, custom domain, or production credentials have been
selected or configured.

## Deployment requirements

The selected provider must:

- build the site with the pinned Rust toolchain, Node.js dependencies and
  `trunk build --release`;
- publish the resulting `dist/` directory;
- rewrite non-file requests to `index.html`, so direct visits to `/projects`
  and `/cv` reach the client-side router;
- serve the site over HTTPS and support a future custom domain;
- keep deployment credentials separate from `PORTFOLIO_GITHUB_TOKEN`.

The GitHub project and CV synchronizers are build-time maintenance tools. They
must not run in the browser or as part of a public deployment runtime. Their
scheduled GitHub Actions workflows update generated source data before the
normal deployment build runs.

## Provider decision

The project should evaluate Cloudflare Pages, Netlify and GitHub Pages when
hosting is ready. Cloudflare Pages and Netlify are the most straightforward
choices for a private repository and SPA rewrite support. GitHub Pages remains
an option only after confirming that private-repository publishing is available
for the account and that its routing configuration meets the application’s
needs.

Choose a provider before adding deployment credentials, domain records or a
deployment workflow. This keeps the repository provider-neutral and avoids
unused secrets.

The same decision must provide the canonical public origin. Only then should
deployment add absolute canonical and Open Graph URLs, and create `robots.txt`
and `sitemap.xml` for the three canonical public routes. The current static
metadata is deliberately site-wide because non-rendering crawlers do not
execute the client application; the selected generic sharing metadata does not
require a rendering strategy.

## Implementation checklist

1. Select the provider and create the hosting project connected to `main`.
2. Configure the production build command as `npm ci && npm run css:build && trunk build --release`.
3. Configure `dist/` as the published directory and add the SPA rewrite rule.
4. Add only the provider-specific secrets required for deployment.
5. Enable preview deployments for pull requests if supported.
6. Configure the custom domain, HTTPS and redirects when the domain is known.
7. Verify direct navigation to `/`, `/projects`, `/cv` and an unknown route.
8. Add a deployment status check or GitHub Actions workflow only after the
   provider integration has been confirmed.

## Operational checks

After deployment, confirm that the daily portfolio synchronization workflow
can merge generated updates without disrupting deployment, and that a `main`
build publishes the refreshed project catalogue. Monitor build failures and
configure notifications through the chosen provider rather than storing a
deployment token in repository files.
