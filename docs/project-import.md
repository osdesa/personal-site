# GitHub project synchronization

The Projects page and the homepage's selected-projects section both consume
`src/generated_projects.rs`. That file is generated at build-maintenance time;
the browser never calls GitHub and never receives an access token.

## Selection

GitHub exposes authenticated user-curated repository lists through the
supported GraphQL API: `User.lists`, `UserList.items`, and the repository-only
`UserListItems` union. The synchronizer therefore treats the named `portfolio`
list as its primary source of truth.

Selection proceeds in strict order:

1. Resolve repositories in the authenticated viewer's named `portfolio` list.
2. If that list is missing or empty, keep accessible `osdesa` repositories with
   the `portfolio` topic.
3. If neither selects anything, use `fallback_repositories` from
   `portfolio-projects.toml`.

General star status is never used, and sources are never merged. A temporary
GraphQL or transport failure stops synchronization and preserves existing data;
it does not silently switch selection sources.

Add or remove projects in the GitHub `portfolio` starred list. The topic and
seeded allowlist (`osdesa/personal-site`, `osdesa/Blocky`) exist only for older
GitHub Enterprise deployments or configurations where user lists are absent.

## Optional repository metadata

Each selected repository may contain `.github/portfolio.toml` on its default
branch. Unknown keys, malformed TOML, invalid dates and unsafe URLs fail the
whole candidate synchronization before the current generated file is touched.

```toml
title = "Display title"
summary = "Short public portfolio summary."
date = "2026-05-04"
status = "Active"
technologies = ["Rust", "Leptos"]
highlights = [
  "A concrete result or engineering decision.",
  "Another concise, publicly shareable point.",
]
image = "https://example.com/project.webp"
demo_url = "https://example.com/demo"
show_repository = true
include_archived = false
include_fork = false
```

All fields are optional. `include_archived` and `include_fork` default to
`false`. Public repositories show their link by default; private repositories
hide it by default. `show_repository` can explicitly override either behavior.
A private project with no link renders a “Private repository” indicator. Text,
technology names, images, highlights and links from private repositories become
public when generated, so metadata must contain only publishable information.

## Normalization rules

- title: metadata title, then the repository name with separators formatted
- summary: metadata summary, GitHub description, then a neutral owner fallback
- technologies: metadata list, otherwise topics followed by primary language
- date: metadata `date`, then GitHub creation date
- demo: metadata `demo_url`, then repository homepage
- image: metadata `image`, then `/images/project-default.svg`
- repository link: GitHub URL for public repositories, hidden for private
  repositories, with `show_repository` as an explicit override
- visibility: GitHub's authenticated public/private value

Blank optional values are omitted. Technology and highlight lists are trimmed
and deduplicated case-insensitively. HTTPS is required for remote repository,
demo and image URLs; a root-relative image path is also accepted.

Archived repositories and forks are excluded unless their own metadata opts
them in. Projects are sorted newest-first by effective portfolio date, then
repository creation date, then case-insensitive full repository name. At most
four are generated. Stars, push time and update time do not affect ordering.

## Authentication and automation

Create a fine-grained personal access token with access only to the candidate
repositories, including private repositories that may be published. Grant:

- Repository permissions: **Metadata — Read-only**
- Repository permissions: **Contents — Read-only**
- Account permissions: **Starring — Read-only**

GitHub documents its [GraphQL user-list schema](https://docs.github.com/en/graphql/reference/users),
Metadata read for [listing authenticated repositories](https://docs.github.com/en/rest/repos/repos#list-repositories-for-the-authenticated-user),
Contents read for [retrieving repository content](https://docs.github.com/en/rest/repos/contents#get-repository-content),
and Starring read for authenticated star operations.
Store the token as the `PORTFOLIO_GITHUB_TOKEN` Actions secret. It must not be
placed in repository files or a browser environment. The workflow's built-in
token (or optional `PORTFOLIO_SYNC_TOKEN`) is separate and only opens the
generated-data pull request.

`.github/workflows/sync-projects.yml` runs daily at 05:41 UTC and on manual
dispatch. It generates all candidate bytes in memory, validates them, performs
an atomic no-op-aware replacement, runs the full Rust quality suite, and opens
or updates `automation/project-sync` only when data changed. A fetch or
validation failure exits before replacement, preserving the previous valid
site data. The resulting pull request triggers normal CI; merging it triggers
the production build path.

Run the synchronizer locally from the repository root:

```text
PORTFOLIO_GITHUB_TOKEN=<fine-grained-token> cargo run --locked --release --features project-sync --bin sync-projects -- --root .
```

In PowerShell:

```powershell
$env:PORTFOLIO_GITHUB_TOKEN = "<fine-grained-token>"
cargo run --locked --release --features project-sync --bin sync-projects -- --root .
Remove-Item Env:PORTFOLIO_GITHUB_TOKEN
```
