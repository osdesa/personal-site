# CV source bundle

This directory contains the versioned source artifacts for the canonical CV release
from [`osdesa/cv`](https://github.com/osdesa/cv):

- `Hayden-Farrell-CV.tex` is the synchronized source artifact.
- `Hayden-Farrell-CV.pdf` is the synchronized downloadable artifact.
- `source-manifest.json` records the upstream tag, immutable commit SHA, byte
  lengths and SHA-256 digests for the source, PDF and generated Rust data.

Stage 2 strictly parses the TeX and transactionally generates
`src/generated_cv.rs`. Stage 3 renders that generated value directly and Trunk
copies this directory so the matching PDF is served at
`/cv/Hayden-Farrell-CV.pdf`.

## Automated synchronization

`.github/workflows/sync-cv.yml` runs daily and can be dispatched manually. It
runs the native Rust synchronizer, verifies the repository, and opens or updates
the fixed `automation/cv-sync` pull-request branch only when a newer semantic
version changes the bundle.

The workflow needs the repository setting that permits GitHub Actions to create
pull requests. Its default `GITHUB_TOKEN` is sufficient for synchronization and
PR creation. An optional fine-grained `CV_SYNC_TOKEN` secret with Contents and
Pull requests write access allows the resulting bot PR to trigger other GitHub
Actions workflows; the scheduled workflow runs the full Rust quality suite
itself in either case.

## Manual operation

From the repository root:

```text
cargo run --locked --release --features cv-sync --bin sync-cv -- --root .
```

`GITHUB_TOKEN` is optional for the public upstream repository but recommended
to avoid anonymous API rate limits. The command prints either the installed
version or an unchanged message. Commit the TeX, PDF, generated Rust module and
manifest together.

## Safety model

The synchronizer:

1. retrieves every upstream tag page and selects the highest valid SemVer tag;
2. uses the commit SHA returned for that tag for both raw downloads;
3. validates the bounded PDF and strictly parses the supported LaTeX grammar;
4. generates a deterministic, statically typed Rust domain value;
5. checks the existing hashes, then reparses and regenerates the local data
   before considering it current;
6. rejects moved tags and semantic-version rollback;
7. stages and flushes all files in the destination filesystem;
8. backs up the current files, installs the manifest last, and restores every
   backup if a replacement operation fails.

Network, API, parsing, validation, staging and lock failures happen before any
committed file is replaced. The last valid checked-in bundle therefore remains
available for review and deployment.

The supported grammar, typed output, presentation contract and format limitations are
documented in [`docs/cv-import.md`](../../docs/cv-import.md).
