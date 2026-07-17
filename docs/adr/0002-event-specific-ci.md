# ADR 0002: Event-specific continuous integration

> Amended by [ADR 0009](0009-ci-bundle-reuse-and-quality-gates.md), which
> defines the current job boundaries and quality gates. The event scope and
> cancellation policy below remain in effect.

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

Use one workflow selected by GitHub event:

- `pull_request` activity targeting `main` runs the complete native, CSS,
  production-bundle and browser-accessibility validation.
- `push` activity on `main` runs the same validation and additionally enforces
  the production Lighthouse budgets.

The pull request trigger listens to opened, reopened, synchronized and
ready-for-review events. Consequently, new feature-branch commits run CI only
when GitHub associates them with an open pull request targeting `main`.

Concurrency is keyed by pull request number for pull request checks, so newer
commits cancel stale work. Main builds are keyed by commit SHA, so every commit
on `main` retains an independent full build.

## Rationale

Restricting branch checks to pull requests focuses CI resources on changes under
review. The main-only performance gate protects the post-integration release
artifact without adding its nine navigations to every review update. ADR 0009
defines artifact reuse so the broader pull-request coverage does not duplicate
release packaging.

## Consequences

- Pull requests receive complete build and browser feedback without the
  main-only performance audit.
- Feature branches without an open pull request receive no remote CI feedback;
  contributors must run the documented checks locally.
- Branch protection should require all pull-request jobs: native quality,
  production WebAssembly bundle and browser accessibility checks.
