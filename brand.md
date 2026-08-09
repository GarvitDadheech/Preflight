# Brand — Preflight

A developer tool for Solana program upgrade safety: replay the same transactions
against two builds of a program and see exactly what changed before deploying.

_Chosen directly (not via the interactive `brand-design` picker, at the user's
request to just execute) on 2026-08-10. Update by re-running `brand-design` or
editing this file and `client/src/index.css` together._

## Palette — Midnight Signal

**Vibe:** technical · serious · trustworthy — "a terminal for money, cold, sharp, never frivolous"
**Category:** infra / dev tooling
**Mood:** technical, serious

Chosen over alternatives like Quantum Lab (too premium/violet-hyped) and Graphite
(too flat/grayscale to carry a "safe vs. unsafe" verdict) because Preflight's core
job is rendering a trust judgment — a cool, restrained blue reads as precise and
audit-like rather than decorative.

### Seeds

| Role | OKLCH | Hex |
|---|---|---|
| bg-base | `oklch(0.14 0.02 265)` | `#0A0E1A` |
| bg-elevated | `oklch(0.19 0.02 265)` | `#141A29` |
| primary | `oklch(0.72 0.17 250)` | `#5B8DEF` |
| primary-soft | `oklch(0.82 0.11 250)` | `#9AB8F5` |
| fg-base | `oklch(0.96 0.01 265)` | `#F1F3F8` |

### shadcn tokens (applied to `client/src/index.css`)

Full token set lives in `:root` (light) and `.dark` in `client/src/index.css`,
inside the `radix-nova` shadcn preset's structure. Theme switching is via the
`.dark` class (see `next-themes` in `client/src/components/theme-provider.tsx`),
defaulting to dark — this is a dark-mode-first developer tool.

**Light mode (`:root`):**

```css
--background: oklch(0.98 0.01 265);
--foreground: oklch(0.18 0.02 265);
--primary: oklch(0.52 0.17 250);
--primary-foreground: oklch(0.98 0 0);
```

**Dark mode (`.dark`):**

```css
--background: oklch(0.14 0.02 265);
--foreground: oklch(0.96 0.01 265);
--primary: oklch(0.72 0.17 250);
--primary-foreground: oklch(0.10 0 0);
```

### Semantic status colors

Preflight's whole UI is about classifying transaction outcomes, so three extra
tokens exist alongside the standard shadcn set, following the same
lightness-shift pattern as `primary` (0.72 dark / 0.52 light) so they sit at a
consistent visual weight in both modes:

| Token | Dark | Light | Used for |
|---|---|---|---|
| `--success` | `oklch(0.72 0.15 150)` | `oklch(0.45 0.13 150)` | unchanged / succeeded |
| `--warning` | `oklch(0.80 0.15 80)` | `oklch(0.48 0.14 65)` | behavior changed, error changed |
| `--info` | `oklch(0.72 0.14 215)` | `oklch(0.50 0.15 225)` | compute units changed |
| `--destructive` (shadcn default) | `oklch(0.65 0.22 25)` | `oklch(0.55 0.22 25)` | new failure, new success (regressions) |

All four are registered in `@theme inline` as `--color-success` etc., so they
work as normal Tailwind utilities: `bg-success/10`, `text-warning`,
`border-destructive/40`.

### Contrast

Palette and derivation follow the shadcn-integration reference's worked example
for Midnight Signal exactly (this palette's dark/light token set was already
contrast-verified there). Semantic tokens reuse `primary`'s exact
lightness values per mode, which passed the same check, just at a different hue.

## Typography — Geist (Pair B)

- **Display + body:** Geist Variable
- **Mono (numbers, addresses, logs, compute units):** Geist Mono Variable

Top pick for "technical · serious" mood, and it's Vercel's own font — a
deliberate nod to the Vercel/Linear/Railway dev-tool reference points this
brand is aiming for.

Wired via `@fontsource-variable/geist` and `@fontsource-variable/geist-mono`
(self-hosted, since this is a Vite app, not Next.js — no `next/font`). Imported
in `client/src/index.css`; CSS variables `--font-sans` / `--font-mono` exposed
via `@theme inline`.

### Type scale

| Role | Class | Use |
|---|---|---|
| Display | `text-4xl font-semibold tracking-tight` | Page hero only |
| H2 (section) | `text-xl font-semibold` | Section breaks |
| H3 (subsection) | `text-sm font-medium` | Card titles |
| Body | `text-sm` | Default UI text |
| Small / caption | `text-xs text-muted-foreground` | Meta, hints |
| Mono | `font-mono tabular-nums` | Program labels, addresses, compute units, logs |

## Gradients

Not used. This palette's mood is technical/serious — gradients would fight the
intent of reading as an audit tool, not a marketing page.

## Tone and voice

### Words to use

Direct, specific, number-forward, factual. State what happened, not how it
feels: "3 transactions that used to succeed now fail" rather than "uh oh,
looks risky!". Prefer nouns and verbs over adjectives.

### Words to avoid

Hype words (revolutionary, supercharge, unleash), exclamation marks, emojis,
overpromising language. Never imply more certainty than the tool actually has
— it replays a fixed example fixture, not arbitrary production traffic, and
copy should keep saying so.

### Voice example

> Succeeded on the old program (value=2000) but failed on the new program:
> instruction 0 failed: custom program error 1 (ValueExceedsMax).

## Usage dos and don'ts

**Do:**

- Use Tailwind's shadcn utility classes (`bg-background`, `text-foreground`,
  `bg-primary`, `text-success`, etc.) everywhere. Never hardcode hex/oklch in
  component files.
- Use `font-mono tabular-nums` for compute unit numbers, safety scores, and
  program labels/addresses.
- Test every component in both light and dark mode (there's a theme toggle in
  the header specifically so this is easy to check while developing).
- Keep motion purposeful: entrance/stagger on load, state changes on
  loading → results, expand/collapse for details. Not decorative animation.

**Don't:**

- Hardcode colors in `.tsx`/`.css` files — fix the token in `index.css` once
  if a color is wrong.
- Use `transition-all` — specify `transition-colors` / `transition-transform`.
- Invent a fifth status color. The four (success/warning/info/destructive)
  cover every `TxOutcomeCategory`; if a new category is added, map it to one
  of these rather than adding a new hue.

---

_Last updated: 2026-08-10. Palette: Midnight Signal · Typography: Geist · Gradients: none._
