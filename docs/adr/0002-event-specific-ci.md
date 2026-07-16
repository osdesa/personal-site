# ADR 0002: Event-specific continuous integration

## Context

The original workflow ran the complete native and WebAssembly production build
for every pull request update as well as every push to `main`. This provided
strong feedback, but repeated the most expensive packaging work before changes
had landed. Running workflows on feature-branch pushes without a pull request
would also consume CI capacity before the branch was ready for review.

## Alternatives

1. Run the complete pipeline for all pushes and pull requests.
2. Run lightweight checks for every feature-branch push and the complete build
   on `main`.
3. Run lightweight checks only for open pull requests targeting `main`, and run
   the complete pipeline for commits pushed or merged into `main`.

## Decision

Use separate jobs in one workflow, selected by their GitHub event:

- `pull_request` activity targeting `main` runs formatting, Clippy and tests.
- `push` activity on `main` additionally installs frontend tooling, generates
  CSS, creates the native release build and creates the production WebAssembly
  bundle.

The pull request trigger listens to opened, reopened, synchronized and
ready-for-review events. Consequently, new feature-branch commits run CI only
when GitHub associates them with an open pull request targeting `main`.

Concurrency is keyed by pull request number for pull request checks, so newer
commits cancel stale work. Main builds are keyed by commit SHA, so every commit
on `main` retains an independent full build.

## Rationale

Formatting, linting and tests provide fast review feedback without performing
release packaging twice. Restricting branch checks to pull requests focuses CI
resources on changes under review. A complete build after changes reach `main`
still verifies the exact integration state intended for deployment.

## Consequences

- Pull requests receive faster feedback and use less CI time.
- Feature branches without an open pull request receive no remote CI feedback;
  contributors must run the documented checks locally.
- CSS generation and release-only packaging failures are discovered after a
  commit reaches `main`, rather than during pull request validation.
- Branch protection should require the `Pull request checks` job before merge.
