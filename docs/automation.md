# Generated content automation

## Publication flow

```text
CV tag or portfolio repository metadata
              |
              v
      scheduled synchronization workflow
              |
              v
  fixed automation branch and one pull request
              |
              v
  required CI and branch protection -> native auto-merge
              |
              v
 Cloudflare Pages deployment from main to production
```

Both workflows are safe to run manually and use separate non-cancelling
concurrency groups, so a later schedule/manual dispatch waits rather than
racing a branch mutation already in progress.

| Workflow | Schedule (UTC) | Source | Branch | Write secret |
| --- | --- | --- | --- | --- |
| `sync-cv.yml` | daily 05:17 | public `osdesa/cv` semantic tags, TeX and PDF | `automation/cv-sync` | `CV_SYNC_TOKEN` |
| `sync-projects.yml` | daily 05:41 | selected GitHub repository metadata and optional portfolio overrides | `automation/project-sync` | `PORTFOLIO_SYNC_TOKEN` |

`PORTFOLIO_GITHUB_TOKEN` is used only by the project workflow to read the
selected public/private repositories. The public CV source does not receive a
read credential. The two `*_SYNC_TOKEN` secrets are used only in the narrowly
scoped steps that identify the writing actor, push the generated branch, create
or update its PR, and enable native auto-merge. No secret is written to source,
placed in a Cloudflare setting, or passed into the Trunk build.

Give the sync tokens the smallest repository scope that permits contents and
pull-request write operations for `osdesa/personal-site`. Give the portfolio
read token only the GitHub read permissions documented in
[`project-import.md`](project-import.md): Starring, Metadata and Contents.
The workflows' built-in `GITHUB_TOKEN` is explicitly read-only; all mutations
use the dedicated write secret in the one step that needs it.

## Expected behavior

The CV synchronizer selects the highest semantic tag, resolves its immutable
commit SHA, validates the expected TeX/PDF filenames and formats, and commits
the source, PDF, generated Rust data and manifest transactionally. Invalid or
missing upstream artifacts fail loudly and preserve the current site content.

The project synchronizer retains the existing selection order: the authenticated
`portfolio` list, then topic, then local allowlist. It filters and normalizes
eligible candidates deterministically and emits at most four projects. Private
repository data is publishable only when intentionally selected; generated text
and links are public even if the source repository is private. Visitors may be
unable to follow an intentionally published private repository link.

Both synchronizers are no-ops when generated output is already current. The
fixed branch and `create-pull-request` action update the existing PR instead of
opening duplicates. Their workflow logs identify discovery, validation,
generation and no-change outcomes without printing credentials.

## Trusted auto-merge policy

The workflows run only from trusted `main` code on a schedule or manual
dispatch; they do not use `pull_request_target` and never execute untrusted fork
code with write credentials. Before requesting GitHub native auto-merge, the
workflow queries the PR it just created or updated and the repository-owned
`verify-auto-merge` tool requires all of the following:

- exact expected branch (`automation/cv-sync` or `automation/project-sync`);
- base branch `main`;
- head repository exactly `osdesa/personal-site`, never a fork;
- PR author matches the authenticated writer token's GitHub identity;
- the corresponding workflow-controlled HTML marker is present in the PR body.

Only then does `gh pr merge --auto --squash` request native GitHub auto-merge.
GitHub keeps the PR unmerged until all required CI checks and branch-protection
requirements pass. Any failed eligibility check, missing repository
auto-merge setting, failing CI check, branch-protection block or merge conflict
fails visibly in the workflow rather than merging around the safeguard.

## Manual operation and recovery

To test a CV sync, open **Actions** > **Synchronize CV** > **Run workflow**,
select `main`, and run it. Review the log for the selected tag and either the
clean no-change message or the generated PR. To test a project sync, use the
same procedure for **Synchronize portfolio projects**; verify the selected
method and project count in the log. In either case, open the fixed-branch PR,
confirm its marker and generated diff, and watch required CI complete before
GitHub merges it and Pages deploys `main`.

If a run fails, use the explicit failed step rather than retrying blindly:

- a missing/expired/revoked token or API rate limit fails the token/API step;
- missing CV tags/assets, unexpected formats, invalid metadata, or private
  access failures fail the synchronizer while preserving current generated data;
- branch push, PR creation, native auto-merge configuration and protection
  blocks fail their own named workflow step;
- CI failures leave the trusted PR open for repair; native auto-merge does not
  bypass them.

Rotate a token by creating a replacement with the same minimal permission,
updating the matching GitHub Actions secret, then manually dispatching the
workflow. Do not add it to Cloudflare. For a stuck fixed branch, first inspect
its open PR and resolve/rebase the conflict through a normal reviewed change;
if no PR is open and the branch is genuinely stale, delete only that explicit
automation branch in GitHub and rerun its workflow. To disable automation
safely, disable the individual workflow in GitHub Actions (or remove its
schedule in a reviewed change); leave the current generated site on `main`
untouched. Re-enable only after correcting the cause.
