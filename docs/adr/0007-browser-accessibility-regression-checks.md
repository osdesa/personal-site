# Browser accessibility regression checks

## Context

Rust integration tests validate content and rendered component contracts, but
they cannot verify browser-only concerns such as computed contrast, landmarks,
keyboard focus, responsive overflow or reduced-motion media queries. The web
quality milestone requires repeatable evidence for the three public routes and
the not-found view without changing the static Leptos/Trunk architecture.

## Alternatives

- Rely on manual checks alone.
- Add broad end-to-end visual snapshots.
- Add a browser accessibility scanner and a small set of focused interaction
  assertions.

## Decision

Use Playwright with `@axe-core/playwright` against a release-style local Trunk
server. The suite scans `/`, `/projects`, `/cv` and an unknown path, then
asserts the 320px mobile navigation's inert closed state, keyboard order, focus
return, lack of horizontal overflow and reduced-motion rules. It runs in both
pull-request and main CI workflows.

## Rationale

This is a maintained, browser-level boundary with a small dependency surface.
axe provides consistent automated coverage for common semantic and contrast
issues, while focused assertions protect the site's bespoke responsive
navigation behaviour. Manual review remains necessary for visual context and
real keyboard flow; snapshots would add brittle maintenance cost without
covering those interactions as directly.

## Consequences

Contributors need Chromium installed through Playwright before running
`npm run test:browser`; CI installs it explicitly. The application keeps no
runtime JavaScript dependency beyond Leptos and its existing assets. Browser
test reports and traces are generated locally only and ignored by Git.
