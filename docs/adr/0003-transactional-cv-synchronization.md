# ADR 0003: Transactional CV source synchronization

- **Status:** Accepted
- **Date:** 2026-07-17

## Context

The canonical CV is maintained in `osdesa/cv`, while this static website needs
a dependable local PDF and will later need the TeX source for derived content.
Copying files manually loses provenance and can expose a partial or invalid
update if one download fails. Reading a mutable tag directly also permits a tag
to move between downloads.

Stage 1 must synchronize the source artifacts without parsing CV content or
changing presentation. The repository remains a client-side Leptos site, so a
runtime backend would add operational complexity without improving this
scheduled build-time concern.

## Alternatives

### Download raw files directly in shell steps

This is short, but semantic ordering, PDF validation, manifest integrity,
rollback and deterministic tests become fragmented across shell dialects.

### Consume the upstream default branch

This always sees recent content, but has no stable release identity and can
observe source and PDF from different commits.

### Depend on GitHub releases

Release assets would provide a suitable boundary, but the upstream repository
currently publishes semantic Git tags rather than GitHub releases.

### Native Rust synchronizer with a transactional local store

A small native module can keep semantic selection and validation independent of
GitHub transport, perform bounded downloads by immutable SHA, and test failure
behavior without a network dependency.

## Decision

Add a native-only `cv_sync` library module and thin `sync-cv` binary to the
existing package. Its application service depends on a small `CvSource` trait;
the production adapter uses GitHub's paginated tag API and raw-content service.

Accept optional `v`-prefixed semantic versions and select by SemVer precedence,
with a deterministic tag-name tie-breaker. Use the commit SHA returned with the
selected tag as the raw download revision. Record that tag, SHA, filenames,
lengths and SHA-256 digests in a strict schema-versioned JSON manifest.

Validate the TeX as bounded UTF-8 with ordered document markers. Validate the
PDF with size bounds, header/trailer checks and a PDF parser requiring at least
one page. This is artifact validation only; Stage 1 does not interpret or render
CV content.

Before replacement, verify any existing manifest and both local artifacts. An
unchanged tag and SHA returns without downloads or writes. Reject a reused tag
that resolves to another SHA and reject upstream version rollback.

For an update, stage and flush all three files beside their destination, acquire
an exclusive lock, move the current files to same-filesystem backups, then
install TeX, PDF and finally the manifest. Any reported replacement failure
rolls back installed files and restores the backups. The manifest is the commit
marker for future consumers.

Run the binary daily in GitHub Actions. Verify formatting, Clippy, tests and the
release binary before a fixed automation branch is created or updated. Keep the
update reviewable through a pull request instead of writing directly to `main`.

## Rationale

- Immutable-SHA downloads prevent a tag race between the two artifacts.
- A strict manifest makes provenance and local corruption auditable.
- Bounded validation rejects common error pages, truncation and malformed PDFs.
- Native-only dependencies do not enter the WebAssembly application.
- The source trait and filesystem store give deterministic failure-path tests.
- A fixed PR branch avoids duplicate daily pull requests.

## Consequences

### Positive

- Network or validation failures leave the current bundle untouched.
- An unchanged release creates no filesystem or Git change.
- Both source files have exact, reviewable upstream provenance.
- Later parsing stages can rely on a validated manifest boundary.
- The existing site runtime and presentation remain unchanged.

### Negative

- Native builds gain a small HTTP client, SemVer, hashing and PDF parser.
- Repository settings must allow Actions to create pull requests.
- Bot-created PRs require an optional non-`GITHUB_TOKEN` credential if they must
  trigger separate workflows; the synchronization workflow therefore performs
  its own quality checks before creating the PR.
- A filesystem cannot expose three independent paths as one indivisible rename;
  the implementation uses staging, same-filesystem backups, rollback and a
  manifest-last commit marker to provide transactional behavior for all
  reported failures.
