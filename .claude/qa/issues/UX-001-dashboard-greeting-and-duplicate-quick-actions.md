---
id: UX-001
title: Dashboard spends half the first screen on a greeting and seven tiles that duplicate the sidebar
status: verified
severity: major
kind: ux
area: dashboard
mode: tauri
found: 2026-11-06
---

## Use case

Open the app and see the state of the workspace.

## Measured

Window 1280x800, `main` = 760px tall (sidebar open → 1024px wide).

| block | px | file |
| --- | --- | --- |
| `{greeting}!` h1 (`text-4xl`) + "Welcome to Portal Desktop…" + `mb-8` | **82** | [src/routes/+page.svelte:140-145](../../../src/routes/+page.svelte#L140) |
| Quick Actions card (header + 7 `h-24` tiles in a 6-col grid) | **299** | [src/routes/+page.svelte:281-300](../../../src/routes/+page.svelte#L281) |
| 4 stat cards | 150 | see UX-002 |

381px of 760 — 50% of the first screen — before the Projects/Tasks summary cards, which
start at y=672 and are the only blocks that say anything about the workspace.

At 1024x600 (minimum window) **nothing** below Quick Actions is on screen, and the tile grid
itself is cut mid-row.

## The seven tiles

`New Project · New Task · Terminal · SDK Manager · Cloud Resources · Credentials · Automation`

Every one is already a left-sidebar entry. Seven of seven are duplicates. The grid is
`lg:grid-cols-6` with 7 items, so the 7th ("Automation") sits alone on a second row.

## Expected

The home screen is the only page with no job of its own, so it should be the densest.

## Actual

Greeting + duplicated navigation fill the fold; the workspace summary is below it.

## Fix

1. Delete the greeting block (`+page.svelte:140-145`). The breadcrumb already says Home.
   If the owner wants a greeting, it belongs as a `text-sm text-muted-foreground` line to
   the right of the stat strip, not as a `text-4xl` h1 with a sentence under it.
2. Delete the Quick Actions card (`+page.svelte:281` through its closing `</Card>`) and the
   `quickActions` array. Keep only the two that are *creations* rather than navigation —
   New Project and New Task — as `size="sm"` buttons in the page header's `actions` snippet.
3. That lifts the Projects / Tasks / Services summary cards to roughly y=190, so four of
   them are visible on the first screen instead of none.

## Evidence

`.claude/qa/shots/ux/ux-home-2.png` (1280x800), `.claude/qa/shots/ux/ux-min1024-home.png`
(1024x600 — zero workspace content on screen).

## Repro

`node .claude/qa/scripts/app.mjs '' ux-home`

## Fix

<!-- coder -->

Root cause: `/` opened with a `text-4xl` greeting and a Quick Actions card that only duplicated
the sidebar, so the workspace summary started below the fold.

- `src/routes/+page.svelte` — greeting block, `quickActions` array and the Quick Actions card
  deleted; `PageHeader title="Overview"` with New Project / New Task as `size="sm"` header
  actions; the four stat Cards replaced with one `PageStats` strip (also closes this page's
  entry in UX-002). `space-y-6` → `space-y-4`.

Measured over CDP at 1280x800, y of the Projects summary card relative to `<main>`:
**637px → 124px**. Evidence: `shots/ux/UX-001-before-home.png`, `shots/ux/UX-001-after-home.png`.

Checks: `pnpm check` exit 0 / 0 ERRORS; `eslint src` 0 errors, `prettier --check src` clean,
shadcn check passed. (`pnpm lint` as a whole exits 1 only on `.claude/handoff.md`, which already
fails `prettier --check` at HEAD and is not part of this change.)

## Verification

<!-- qa-tester -->
**2026-11-07 · tauri (real backend, CDP :9222) · 1280x800 and 1024x600, dark + light**

- Greeting block and Quick Actions card are gone: `main` matches neither
  `/Good (morning|afternoon|evening)|Welcome to Portal Desktop/` nor `/Quick Actions/`.
  Exactly one `h1` on the page: `Overview`.
- Projects summary card now sits at **y=124** below `<main>` (was 637). The stat strip and
  all four summary cards are above the fold at 1280x800.
- Header actions work: `New Project` → `/projects/create` (h1 "Create New Project"),
  `New Task` → `/tasks/create`. Both resolve to real `<button>` elements (tagName
  asserted, so no `role="button"` wrapper was hit).
- **The "all seven tiles duplicate the sidebar" claim was checked, not taken.** Old tile
  targets were `/projects/create /tasks/create /terminal /sdk /cloud /credentials
  /automation`. Enumerating every `a[href]` and `button` outside `main` shows the sidebar
  carries `/terminal`, `/sdk`, `/cloud`, `/credentials` and `/automation`; the two
  creation routes are the new header buttons. Nothing became unreachable.
- 1024x600: no horizontal overflow, header row 32px, both buttons inside the content edge.
  Light theme correct. Console errors: none.

Evidence: `.claude/qa/shots/ux/UX-001-after-home-verify.png`,
`.claude/qa/shots/ux/UX-001-after-home-light.png`,
`.claude/qa/shots/ux/UX-002-after-home-1024x600.png`.
