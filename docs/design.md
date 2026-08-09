# Design system

Locked reference for UI work going forward — check here before inventing a new
color, spacing value, or component pattern. Update this file when a real
design decision changes, not on every tweak.

## Palette

Dark by default (`color-scheme: dark`), no light mode built — this is a
deliberate single-look commitment, not an oversight.

| Token | Value | Use |
|---|---|---|
| `--bg` | `#111113` | Page background |
| `--sidebar-bg` | `#17171a` | Sidebar background |
| `--surface` | `#1b1b1e` | Cards, tiles, inputs — separated from bg by lightness shift, never a border |
| `--surface-hover` | `#232326` | Hover state for surfaces |
| `--fg` | `#ededef` | Primary text |
| `--muted` | `#8b8d93` | Secondary text, labels, inactive nav |
| `--accent` | `#e0a446` | Amber — active nav, links, primary actions. Chosen deliberately over the default AI-generated blue/purple; fits a "vault" mental model |

## Typography

System font stack only — `-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif`.
No webfont import: keeps the app fully self-contained with zero CDN calls,
consistent with the self-hosted, no-external-dependency ethos of the whole
project. Distinctiveness comes from weight and spacing, not font choice.

- Headings (`h1`, panel `h2` labels): 600 weight, small size, `0.02–0.05em`
  letter-spacing.
- Body/UI text: default weight, 0.85–0.9rem for most controls.

## Spacing

8px scale (4px half-step), consistent across the app:

```
--space-1: 4px   --space-3: 16px   --space-5: 40px
--space-2: 8px   --space-4: 24px
```

Proximity carries meaning: space inside a component < space between
components < space between sections.

## Radius

`--radius: 10px` everywhere — tiles, cards, inputs, buttons. One value, no
per-component variation.

## Component patterns

- **Cards/tiles separate by lightness, never by border.** `var(--surface)` on
  `var(--bg)` is the whole separation mechanism. A gray border is the #1
  AI-slop tell — don't add one.
- **Hover feedback**: subtle background shift (`--surface` → `--surface-hover`)
  plus, for tiles, a small `translateY(-2px)` lift. Never a color-only change
  with no motion, never an aggressive scale.
- **Forms are native, unstyled-by-a-library HTML**: `<select>`/`<input>`
  styled directly with the token set above, no design-system dependency.
  Auto-submitting selects (`onchange="this.form.submit()"`) are the standard
  pattern for single-choice actions (add to collection, assign owner) — no JS
  framework, no fetch calls, just a form.
- **Chips** (`​.chip`): `var(--surface)` pill, label + inline remove button,
  used for collection membership.
- **Sidebar nav**: active item gets `var(--surface)` background + `var(--accent)`
  text + bold. Inactive items are `var(--muted)`, hover to `var(--fg)`.
- **Collapsible sidebar**: state persisted in `localStorage`, applied via a
  class on `<html>` set by a blocking inline `<script>` in `<head>` — avoids a
  flash of the sidebar before JS runs. Animate `width`/`opacity`/`padding`,
  never `display: none` (no animation possible on that).
- **Fullscreen/lightbox**: toggled by a CSS class on the existing element
  (`position: fixed; inset: 0`), not a separate overlay element — one image,
  one state toggle, closes on click-outside or Escape.
- **Empty states**: always a muted, centered one-liner (`.empty`), never a
  bare empty grid.

## What we deliberately did NOT do

- No CSS framework (Tailwind, Bootstrap) — the token set above is small enough
  to hand-write and stay consistent without one.
- No JS framework — vanilla `<script>` blocks, plain forms, no build step.
  Matches the htmx-adjacent "server-rendered, sprinkle JS only where needed"
  architecture decision from the main spec.
- No animation library — CSS transitions only, all under 0.2s.
