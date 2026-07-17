# ADR 0006: Generate portfolio projects from GitHub metadata

## Context

Portfolio project cards were manually maintained samples. The intended
selection lives in a named GitHub starred list, must support private sources,
and must not expose GitHub credentials in the static browser application.
GitHub's GraphQL schema exposes user-curated repository lists and their items.

## Alternatives

1. Query all stars in the browser and filter them heuristically.
2. Scrape the named-list website UI.
3. Maintain only a local repository allowlist.
4. Use the authenticated GraphQL user-list API, retain topic and allowlist
   fallbacks, and generate typed Rust data in Actions.

## Decision

Use option 4. A native application service resolves the authenticated viewer's
named `portfolio` list. A missing or empty list falls back first to the
`portfolio` topic and then the repository-local allowlist. It retrieves GitHub
metadata and optional strict
`.github/portfolio.toml` content with a fine-grained read token, normalizes the
complete set, filters archived repositories and forks, sorts and caps it, then
atomically replaces a deterministic Rust module only when bytes differ.

Both website surfaces consume the same generated slice. GitHub transport,
TOML parsing, filesystem mutation and credentials are native-only.

## Rationale

GraphQL user lists preserve the existing GitHub selection experience and work
for public and private repositories visible to the token. Topics and the
allowlist keep older or list-free configurations operable without scraping.
Static Rust data follows the established CV pipeline, adds no browser parsing,
and catches schema errors at compile time.

## Consequences

- Repository selection remains managed through the named GitHub starred list.
- A fine-grained secret with Starring, Metadata and Contents read is required
  for private project synchronization.
- Any generated information about a private repository is public website data.
- Remote or validation failure leaves the previous generated module intact.
- Project changes arrive through an automation pull request and normal CI.
