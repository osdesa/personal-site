# Design system

## Direction

The interface is modern, technical, calm and deliberate. Visual character comes
from strong type scale, precise alignment, neutral grey contrast, fine grid
lines and geometric project marks—not decorative animation or generic visual
effects.

The site uses one permanent dark appearance. It has no selector, light-mode
token set or browser theme persistence.

The homepage opens with a viewport-scale introduction: oversized name,
one-line role and summary, two actions, and plain professional links over a
restrained gradient. The page contains no profile imagery.

## Tokens

All tokens are declared at the start of `styles/input.css`. Tailwind's `@theme`
defines primitive values; semantic CSS variables expose the fixed palette to
components.

### Colour

| Role | Value |
| --- | --- |
| Canvas | `#111214` |
| Subtle surface | `#17191c` |
| Raised surface | `#1c1f23` |
| Primary text | `#f1f2f2` |
| Secondary text | `#b2b5b8` |
| Primary accent | `#c6cdd4` |
| Focus | `#dce2e7` |
| Border | `#2c3035` |

Components use semantic variables such as `--surface-raised`, `--text-primary`
and `--accent`; page modules must not introduce one-off colours.

### Typography

- UI/body: Inter-compatible system sans stack, avoiding an external font request
- Technical labels: `SFMono-Regular`, Consolas or system monospace
- Headings: weight 650, tight tracking and compact line height
- Body: 1rem base size and 1.65 line height
- Eyebrows: uppercase monospace at 0.72rem with wide tracking

Fluid `clamp()` scales keep headings proportional between mobile and desktop.

### Spacing and width

- Base rhythm is quarter-rem compatible, with component padding from 0.75rem to
  2.5rem.
- Section spacing uses `clamp(5rem, 9vw, 8.5rem)`.
- Content is capped at 76rem with 1–1.25rem mobile gutters.
- Narrow prose is capped near 48rem.

### Shape, elevation and movement

- Small controls: 0.5rem radius
- Content cards: 1.25rem radius
- Major panels: 1.75rem radius
- Raised cards use one dark-surface, low-spread shadow token
- Fast feedback: 160ms ease
- Layout/hover movement: 420ms exponential ease
- `prefers-reduced-motion` effectively removes transitions and smooth scrolling

## Responsive rules

The design responds by content needs rather than device names:

- above 64rem: full desktop composition
- at 52rem: mobile navigation, single-column hero/interior layouts
- at 40rem: single-column project grids, full-width calls to action and compact
  footer arrangement
The homepage hero uses additional height queries to reduce type size and
supporting text on short viewports.

All grids use `minmax(0, ...)`, flexible wrapping and bounded containers to
prevent horizontal overflow. Controls retain comfortable touch targets.

## Components

### Navigation

Desktop links show the active route with colour and an underline. Mobile uses an
explicitly labelled button with `aria-expanded`; its closed list is visually
hidden and natively inert, so its links are absent from both the accessibility
tree and keyboard order. Selecting a mobile route closes the menu and moves
focus to the focusable main landmark. The sticky header maintains context.

### Buttons and text links

Use the primary button for the dominant page action and the secondary button for
an alternative. Links are always anchors. Disabled actions are rendered as
non-interactive elements with `aria-disabled`, never clickable placeholders.

### Project cards

Cards contain artwork, visibility/date/status metadata, heading, summary,
technology badges, optional highlights and explicit repository/demo links.
Optional elements are omitted rather than rendered empty. A hidden private
repository uses a clear non-link indicator. Artwork is controlled local content
with explicit dimensions and a reserved 608:272 aspect ratio. It is decorative;
if it fails or is slow, the title and written project context remain available.

### Section headings

Every major section uses an eyebrow, level-two heading and optional supporting
copy. Pages preserve one level-one heading and a logical level hierarchy.

### Generated CV

The CV follows the balanced two-column layout established in commit `67cbe097`:
a section-heading rail beside a wide reading column. At 52rem it becomes one
column. Experience and education use the original vertical timeline treatment,
with decorative markers, compact date labels, and organisation/location grouped
beneath each heading. Skills reuse the shared badge language. Projects are not
repeated on the CV page because they have a dedicated route. The web hierarchy
prioritises readable content flow and does not imitate the PDF's print layout.

Imported inline emphasis maps to native semantic elements. Imported links keep
the standard focus ring and a subtle token-based underline. Source version and
short commit provenance appear in a quiet closing section without competing
with the download action.

### Focus and accessibility

- Every keyboard-focusable element receives a 3px high-contrast focus ring.
- A skip link targets the focusable main element.
- Native landmarks, lists, headings, buttons and anchors are preferred to ARIA.
- New-tab context is included in accessible link names.
- Decorative geometry is hidden from assistive technology.
- Text and controls use fixed dark-palette values selected for clear contrast.
- Browser checks scan every public and not-found view with axe, and exercise the
  320px mobile menu, overflow prevention and reduced-motion contract.

## Adding styles

1. Reuse an existing semantic token or component rule.
2. Add a new token only when the value represents a repeated design decision.
3. Keep route-specific layout in a small semantic class, not inline style.
4. Check the permanent dark palette, keyboard focus, reduced motion and widths from
   320px through wide desktop.
5. Run `npm run css:build` before the Wasm production build.
