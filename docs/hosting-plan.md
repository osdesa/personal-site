# Hosting plan

## Purpose and constraints

This plan publishes the existing client-side Leptos application without
changing its static architecture. The production artifact is Trunk's `dist/`
directory; the native `personal-site` binary and the `sync-cv` and
`sync-projects` maintenance binaries are not deployed.

The plan preserves the following repository contracts:

- `main` is the production source branch and pull requests must continue to
  pass the existing CI quality gates before merging;
- the deployed host must return `index.html` for client routes, including
  `/projects`, `/cv`, and unknown paths, while serving real assets such as the
  CV PDF unchanged;
- production remains free of runtime secrets, server processes, third-party
  scripts, remote fonts, and runtime GitHub calls;
- `PORTFOLIO_GITHUB_TOKEN`, `CV_SYNC_TOKEN`, and `PORTFOLIO_SYNC_TOKEN` remain
  GitHub Actions-only secrets for scheduled source synchronization;
- generic, site-wide sharing metadata remains the selected approach. Canonical
  URLs, `robots.txt`, and `sitemap.xml` wait until the final public origin is
  known.

After the one-time hosting and automation setup, normal content maintenance
must not require a person to edit, approve, or merge changes in this website
repository. The CV release repository and GitHub portfolio metadata are the
owner-maintained inputs; this repository is their generated publication target.

## Recommendation

Use **Cloudflare Pages with GitHub integration** for the initial host. It is a
good fit for this small static bundle: it supports a Git-connected production
branch and preview deployments, deploys a configured build output directory,
and applies SPA fallback behaviour when no top-level `404.html` is present.
That fallback lets Leptos Router render the existing Not Found page rather than
requiring provider-specific route files.

Use the repository root as the Pages root, set `main` as the production branch,
and allow preview deployments only for pull requests. Do not add Pages
Functions, Workers, analytics, or a deployment token in this phase.

Every merge into `main` must automatically create a production deployment. The
GitHub integration performs this from the merged commit; no manual deployment
command or repository deployment secret is needed.

Cloudflare Pages is the preferred choice, not a permanent architectural
commitment. Netlify remains the fallback if a build-environment validation
spike shows that Pages cannot reliably provide the pinned Rust, WebAssembly,
Trunk, and Node toolchain. GitHub Pages is not preferred because its SPA route
fallback needs more bespoke handling.

## Step-by-step Cloudflare Pages runbook

Complete these steps in order. They create hosting configuration only; they do
not change the application until the domain step explicitly calls for it.

### 1. Prepare the repository

1. Ensure the intended release commit is on `main` and its GitHub `CI` workflow
   is green.
2. Confirm the local production build still succeeds:

   ```text
   npm ci
   npm run css:build
   trunk build --release
   npm run test:browser:dist
   npm run test:performance
   ```

3. Do not commit `dist/`, `node_modules/`, Lighthouse reports, a Cloudflare API
   token, or any synchronization token. The host builds `dist/` from the
   committed source.

### 2. Create and validate a preview project

1. Sign in to Cloudflare and open **Workers & Pages**.
2. Select **Create application** > **Pages** > **Import an existing Git
   repository**.
3. Authorize the Cloudflare GitHub App for this repository only, then select
   the `personal-site` repository.
4. In the build configuration screen, set:

   | Field | Value |
   | --- | --- |
   | Framework preset | None / custom |
   | Production branch | `main` |
   | Root directory | leave empty (repository root) |
   | Build output directory | `dist` |
   | Node.js version environment variable | `NODE_VERSION=24` |
   | Build command | `rustup target add wasm32-unknown-unknown && cargo install trunk --version 0.21.14 --locked && npm ci && npm run css:build && trunk build --release` |

5. Select **Save and Deploy**. Wait for the first deployment to finish, then
   open the generated `*.pages.dev` preview URL.
6. If the build fails because the Pages image cannot provide the Rust toolchain
   required by `rust-toolchain.toml`, stop here. Capture the build log and use
   the fallback evaluation in the plan; do not loosen the pinned toolchain
   merely to make the first deployment pass.

### 3. Test routing and static assets on the preview

Open each URL directly in a new browser tab, then refresh it:

```text
https://<pages-project>.pages.dev/
https://<pages-project>.pages.dev/projects
https://<pages-project>.pages.dev/cv
https://<pages-project>.pages.dev/a-route-that-does-not-exist
https://<pages-project>.pages.dev/cv/Hayden-Farrell-CV.pdf
```

Expected results:

- the first three URLs render their normal Leptos pages;
- the unknown URL renders the application's Not Found page, not a Cloudflare
  error page;
- the PDF downloads or opens as a PDF, not as the application's HTML;
- `favicon.svg` and project images load successfully;
- browser DevTools shows no failed Wasm, JavaScript, CSS, or asset requests.

Do not add a `404.html` file. Its presence would change Pages from its default
SPA fallback behaviour. Do not add a broad `_redirects` rule unless this test
fails and the provider behaviour has been rechecked.

### 4. Configure pull-request previews and merge protection

1. In the Pages project's **Settings** > **Builds**, set `main` as the
   production branch and configure preview branch controls to include pull
   request branches only.
2. In GitHub, require the existing `CI` workflow before merges to `main`.
   Pages deployment status can be required too once it is stable, but it must
   not replace `CI`.
3. Open a small documentation-only pull request. Confirm it receives a Pages
   preview URL, and that merging it causes a fresh production deployment from
   `main`. Verify the deployment identifies the merge commit, finishes
   successfully, and updates the canonical Pages URL.
4. Do not expose `PORTFOLIO_GITHUB_TOKEN`, `CV_SYNC_TOKEN`, or
   `PORTFOLIO_SYNC_TOKEN` as Pages environment variables. They belong only to
   the scheduled GitHub Actions workflows.

### 5. Attach the custom domain

1. Decide the canonical hostname (for example, `www.<domain>` or `<domain>`)
   before editing DNS. Record that choice in the hosting issue or pull request.
2. In **Workers & Pages** > the Pages project > **Custom domains**, choose
   **Set up a custom domain** and enter the canonical hostname.
3. Complete the DNS records and domain verification shown by Cloudflare. Wait
   until the domain is active and the browser presents a valid HTTPS
   certificate.
4. Add the other hostname as an additional domain and configure a permanent
   redirect to the canonical HTTPS hostname. Preserve the requested path.
5. Repeat the route and asset tests from Step 3 using the canonical HTTPS URL.

### 6. Make the one required application follow-up

After the final domain works, open a normal pull request that:

1. adds one canonical-origin configuration value;
2. generates absolute canonical and Open Graph URLs from that one value;
3. adds `public/robots.txt` and `public/sitemap.xml` with only `/`,
   `/projects`, and `/cv` under the canonical origin;
4. updates the README, deployment notes, web-quality operations, and
   architecture documentation with the chosen provider and domain;
5. adds focused tests for the generated absolute URLs and static files.

Do not introduce server rendering merely for social previews. Retain the
generic static sharing metadata decided in ADR 0008.

### 7. Perform the launch check

1. Run the deployed keyboard, mobile, zoom, reduced-motion, and accessibility
   checks on `/`, `/projects`, `/cv`, and an unknown route.
2. Run Lighthouse against the canonical URL and compare it with the repository
   budgets: performance at least 0.90; accessibility, best-practices, and SEO
   at 1.00; transferred bytes at most 550,000; cumulative layout shift at most
   0.05.
3. Inspect the initial HTML with JavaScript disabled. Confirm generic sharing
   metadata, canonical URLs, `robots.txt`, and `sitemap.xml` all name the same
   HTTPS origin.
4. Merge a generated CV or project synchronization pull request and confirm
   its `main` deployment publishes the new generated content and CV PDF.
5. Publish the canonical URL only after all checks pass.

### 8. Make content publication fully automatic

The existing daily synchronizers already discover upstream CV and project
changes and create reviewable generated-content pull requests. Complete this
one-time setup so those pull requests finish the publication flow without a
person interacting with this repository:

1. Enable GitHub's repository options that allow GitHub Actions to create pull
   requests and allow auto-merge for pull requests that meet branch protection
   requirements.
2. Configure the existing `CV_SYNC_TOKEN` and `PORTFOLIO_SYNC_TOKEN` as
   narrowly scoped bot credentials. They must be able to create the automation
   pull requests and allow their pull-request CI to run; they are distinct from
   the read-only `PORTFOLIO_GITHUB_TOKEN` used to read project data.
3. Add a narrowly scoped auto-merge step to each synchronization workflow. It
   may auto-merge only its own fixed branch (`automation/cv-sync` or
   `automation/project-sync`) and only after all required `CI` checks pass.
   It must never auto-merge arbitrary contributor pull requests.
4. Retain the daily schedules (CV at 05:17 UTC and projects at 05:41 UTC).
   A source change is therefore normally visible within 24 hours, then after
   synchronization, CI, merge, and the automatic Pages deployment complete.
5. Configure GitHub and Cloudflare failure notifications. Normal updates need
   no attention; a failed upstream parse, expired token, or failing quality
   gate is exceptional and must be repaired before publication can resume.

The resulting operational flow is:

```text
Update tagged CV release or portfolio-visible GitHub metadata
                         |
                         v
              daily GitHub synchronization
                         |
                         v
              generated automation pull request
                         |
                         v
              required CI passes -> scoped auto-merge to main
                         |
                         v
              Cloudflare Pages automatically deploys main
```

For a CV update, publish a newer semantic-version tag in `osdesa/cv`; changing
an untagged commit is intentionally ignored. For a project update, change its
portfolio-visible GitHub list/repository fields or `.github/portfolio.toml`.
Ordinary source-code-only changes are not portfolio content and do not alter
the generated site catalogue.

## Phase 0: owner decisions and preflight

1. Choose and register the canonical domain. Decide whether `www` or the apex
   domain is canonical, with the other permanently redirected to it.
2. Create a Cloudflare account and limit the Cloudflare GitHub App to this
   repository only.
3. Run a disposable Pages preview build before adding DNS. Its build must use
   the repository's pinned inputs and complete:

   ```text
   npm ci
   npm run css:build
   trunk build --release
   ```

4. Confirm the build image honours `rust-toolchain.toml`, can install or use
   Trunk `0.21.14`, and has Node.js 24. Record the smallest provider-specific
   configuration needed to do this (for example, a Node version setting and
   an explicit Trunk installation step). If the result cannot be made stable,
   stop and use the Netlify fallback or choose an artifact-deployment design in
   a separate ADR.
5. Confirm the preview's generated `dist/` directory contains `index.html`,
   hashed Wasm and JavaScript assets, stylesheet, `/cv/Hayden-Farrell-CV.pdf`,
   and controlled project images.

## Phase 1: create the host project

1. Create a Pages project connected to the repository. Configure:

   | Setting | Value |
   | --- | --- |
   | Production branch | `main` |
   | Root directory | repository root |
   | Build command | the validated Phase 0 command sequence |
   | Build output directory | `dist` |
   | Preview deployments | pull-request branches only |

2. Do not add deployment secrets unless the validated provider build truly
   requires one. A public static site should not need a deployment token or
   any GitHub synchronization token.
3. Keep the Pages project free of Functions and a top-level `404.html`, so
   the provider's SPA fallback reaches the Leptos application. Do not add a
   catch-all `_redirects` rule unless a deployed verification proves it is
   necessary; it could otherwise interfere with asset responses.
4. Require the existing GitHub `CI` check before merging into `main`. Treat the
   Pages deployment status as an additional delivery signal, not a replacement
   for the repository's Rust, browser-accessibility, and Lighthouse gates.

## Phase 2: domain, HTTPS, and metadata

1. Attach the chosen canonical domain and configure DNS according to
   Cloudflare's domain-validation instructions. Verify that HTTPS is active
   before making the domain public.
2. Add one typed/configured canonical-origin value to the application. Use it
   as the single source for absolute canonical and Open Graph URLs; do not
   duplicate the domain in pages or generated content.
3. Add `robots.txt` and `sitemap.xml` to the static assets after the origin is
   final. The sitemap contains exactly `/`, `/projects`, and `/cv`; it must not
   include the client-side not-found route or synchronization artifacts.
4. Retain the truthful generic text social metadata selected by ADR 0008. A
   later social-card image or route-specific previews require an explicit
   product decision and, for route-specific previews, a new rendering ADR.
5. Redirect every non-canonical hostname and HTTP request to the canonical
   HTTPS origin. Preserve paths and query strings where supported.

## Phase 3: production verification and release

Run this verification on the preview URL, then again on the canonical HTTPS
origin before announcing the site:

1. Directly open and refresh `/`, `/projects`, `/cv`, and an unknown path.
   The three public routes must load and the unknown route must show the app's
   Not Found page; no route may produce a host 404.
2. Confirm `/cv/Hayden-Farrell-CV.pdf`, `/favicon.svg`, and a project image are
   served as assets rather than rewritten to HTML.
3. Run keyboard, 320px mobile, zoom, reduced-motion, and browser accessibility
   checks on the deployed URL, matching the scope of the existing Playwright
   suite.
4. Run the Lighthouse audit against the deployed canonical origin. Compare it
   with the documented local baseline and budgets: performance at least 0.90,
   accessibility/best-practices/SEO at 1.00, transfer at most 550,000 bytes,
   and cumulative layout shift at most 0.05. Record any material host-specific
   variance and its cause.
5. Inspect initial HTML with JavaScript disabled or a crawler-style request.
   Verify generic metadata is present and canonical, Open Graph, `robots.txt`,
   and sitemap URLs are absolute and use the same origin.
6. Merge one scheduled CV or project synchronization pull request after launch
   and verify that its normal `main` deployment publishes the updated generated
   content and matching PDF without a runtime GitHub request.

## Operations, security, and rollback

- Enable provider deployment notifications and retain deployment history.
  Investigate failed builds through the provider logs; do not place deployment
  credentials in repository files.
- Review the Cloudflare GitHub App's repository scope and Pages configuration
  whenever ownership or hosting changes. Keep GitHub Actions tokens scoped to
  their documented read/write purpose and separate from hosting.
- Let the host's normal static-asset caching operate initially. Do not add
  custom cache rules until deployed measurements demonstrate a need, because
  stale Wasm or HTML can break a new release.
- Roll back by selecting the last known-good Pages deployment. If the issue is
  source-related, revert it through a normal pull request so Git history,
  CI, and the deployed revision converge again.
- Revisit this plan only when a genuine server concern appears (for example,
  an authenticated contact endpoint or dynamic content). That change requires
  a new architecture decision record rather than attaching ad-hoc Functions
  to the static host.

## Completion criteria

Hosting is complete when the canonical HTTPS domain serves the validated
`main` bundle, all direct client-route and static-asset checks pass, provider
previews are available for pull requests, domain-dependent metadata is
consistent, every merge into `main` automatically deploys that merge commit,
and a post-deployment quality audit is recorded. Update
`docs/deployment.md`, `docs/web-quality.md`, `docs/architecture.md`, and the
README with the final domain and confirmed provider behaviour at that time.

## Sources

- [Cloudflare Pages Git integration](https://developers.cloudflare.com/pages/get-started/git-integration/)
- [Cloudflare Pages SPA serving behaviour](https://developers.cloudflare.com/pages/configuration/serving-pages/)
- [Cloudflare Pages build configuration](https://developers.cloudflare.com/pages/configuration/build-configuration/)
