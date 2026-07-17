# CV source bundle

This directory contains an opaque, versioned copy of the canonical CV release
from [`osdesa/cv`](https://github.com/osdesa/cv):

- `Hayden-Farrell-CV.tex` is the synchronized source artifact.
- `Hayden-Farrell-CV.pdf` is the synchronized downloadable artifact.
- `source-manifest.json` records the upstream tag, immutable commit SHA, byte
  lengths and SHA-256 digests for both artifacts.

Stage 1 does not parse the TeX into website content or add a new CV viewer. The
existing site continues to copy the PDF as a static download.

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
version or an unchanged message. Commit all three bundle files together.

## Safety model

The synchronizer:

1. retrieves every upstream tag page and selects the highest valid SemVer tag;
2. uses the commit SHA returned for that tag for both raw downloads;
3. validates bounded UTF-8/TeX structure and parses the bounded PDF;
4. checks the existing manifest and file hashes before considering it current;
5. rejects moved tags and semantic-version rollback;
6. stages and flushes all files in the destination filesystem;
7. backs up the current files, installs the manifest last, and restores every
   backup if a replacement operation fails.

Network, API, parsing, validation, staging and lock failures happen before any
committed file is replaced. The last valid checked-in bundle therefore remains
available for review and deployment.
