---
id: UX-005
title: Settings nav is 993px tall for 14 sections, and every page states its title twice
status: verified
severity: major
kind: ux
area: settings
mode: tauri
found: 2026-11-06
---

## Use case

Open Settings and go to the section you want.

## Measured (1280x800, dark theme)

| thing | measured |
| --- | --- |
| settings top bar (h1 "Settings" + description + a `min-h-[2.75rem]` badge row) | **154px** before anything |
| nav Card | 278 x **993px** — it scrolls inside a 622px column |
| nav item height | **71px** each (icon + label + description, two lines) |
| items visible without scrolling | 8 of 14 top-level sections |
| content column | `max-w-readable` = 768px inside an 870px column |

The nav descriptions restate the labels:

```
General   / Application preferences        Terminal / Terminal configuration
Updates   / Check for updates              Theme    / Appearance & colors
Editor    / Code editor settings           Autonomy / Autonomous action settings
```

Then every settings page repeats its own title: the top bar says
**Settings / Configure your application preferences and behavior** and the page body says
**General Settings / Configure application preferences and general behavior**. Two headings
and two near-identical descriptions for one screen, ~75px.

The badge row shows `Configured: 4/7` and `Language: en`. `4/7` is computed from a hardcoded
`const total = 7` whose own comment lists eight sections
([src/routes/settings/+layout.svelte:144-155](../../../src/routes/settings/+layout.svelte#L144)) —
it is not a number anyone can act on.

## Expected

All 14 sections reachable without scrolling the nav; the page title stated once.

## Actual

Nav scrolls at 1280x800; title stated twice on all 16 settings routes.

## Fix

1. [src/lib/domains/settings/components/SettingsNavigation.svelte](../../../src/lib/domains/settings/components/SettingsNavigation.svelte) —
   drop the `description` line from the rendered item (keep the field if a tooltip wants it).
   One line, icon + label, `h-8`. 14 × 36px ≈ 500px: the whole nav fits with no scroll, and
   the Card can lose its `p-3` wrapper.
2. [src/routes/settings/+layout.svelte:206-213](../../../src/routes/settings/+layout.svelte#L206) —
   delete the badge row (and `getSettingsStats`). It reserves 44px whether or not it has
   anything to say.
3. [src/routes/settings/+layout.svelte:171-178](../../../src/routes/settings/+layout.svelte#L171) —
   collapse the h1 + description into one row, and **delete the per-page `<h2>` + description**
   in each settings component (`GeneralSettings.svelte`, `IdeSettings.svelte`, … 16 files);
   instead feed the active section's label into the layout h1 as
   `Settings › General`. The left nav already shows which section is active.
4. Narrow the nav Card to `w-56` (224px). The descriptions were the only reason it was 278.

Net: ~160px of vertical chrome per settings page, plus the nav stops scrolling.

## Evidence

`.claude/qa/shots/ux/ux-settings-general.png`, `.claude/qa/shots/ux/ux-settings-ides.png`,
`.claude/qa/shots/ux/ux-min1024-settings-general.png`.

## Repro

`node .claude/qa/scripts/app.mjs settings/general ux-settings-general`

## Fix

<!-- coder -->

Partial — fix item 3's second half only, under the UX-015 route pass: the per-page `<h2>` +
description is deleted from all 16 settings route pages (`src/routes/settings/**/+page.svelte`),
so each settings screen states its title once. Measured on `/settings/general` at 1280x800:
first real content card moved from y=286 to y=202 (-84px); headings on the screen went from
`["Settings", "General Settings"]` to `["Settings"]`.

Still open here: items 1, 2 and 4 — the nav item descriptions / `h-8` rows, the `w-56` nav card,
and the `Configured: 4/7` badge row with `getSettingsStats`. Left alone deliberately (separate
nav-sizing pass). Headings rendered by settings *components* (`IdeSettings.svelte`'s
"IDE Configuration", `AIProviderSettings`' "AI Provider Configuration") also remain; only the
route-level duplicate was removed.

## Verification

2026-11-07 — tauri, dev instance 2 (vite :1421 / CDP :9223, own database).
Ran `node .claude/qa/scripts/ux005-verify.mjs`, `ux005-verify2.mjs`, `ux005-verify3.mjs`,
`ux005-verify4.mjs`.

Measured on `/settings/general`:

| | before | now |
| --- | --- | --- |
| nav Card | 278 x 993px, scrolled | **288 x 594px** (182px wide at 1024), no inner scroll |
| nav item height | 71px | **32px** top-level, 38px sub-items |
| sections reachable without scrolling at 1280x800 | 8 of 14 | **14 of 14** (last item bottom y=756 < 800) |
| `Configured: 4/7` badge row | present | **gone** (no `Configured: n/m` anywhere in the body) |
| `h1` count on the page | 2 | **1** |

- All 14 labels render on one line and are visible: General, Updates, Editor, Terminal, IDEs,
  Languages, Package Managers, Frameworks, GitHub, Theme, Learning, Providers, Training Data,
  Autonomy, under the two group headers.
- The layout `h1` names the active section from `activeSectionLabel(pathname)` on every route I
  tried, grouped ones included: `Settings › General`, `Settings › Updates`, `Settings › IDEs`,
  `Settings › Autonomy`, `Settings › Theme`, `Settings › AI › Providers`,
  `Settings › Frameworks & Languages › Languages`.
- Keyboard: the items are real `<button>`s, focusable with a visible 1px outline; `Enter` on
  "Updates" navigated to `/settings/updates` and the h1 followed. `aria-current="page"` is on the
  active top-level item.
- At 1024x600 the nav keeps the same 14 one-line items at 182px wide; the column scrolls, which a
  600px-tall window makes unavoidable for a 594px card.

Shots: `.claude/qa/shots/UX-005-after-settings-general-1280.png`,
`.claude/qa/shots/UX-005-after-settings-nav-1024x600.png`,
`.claude/qa/shots/UX-005-after-settings-nav-appsidebar-collapsed.png`.

Two things this verification could not close, filed/noted separately:

- The five sub-section items (Languages, Package Managers, Frameworks, Providers, Training Data)
  carry no `aria-current`, so on `/settings/ai/providers` and `/settings/languages` nothing on the
  page is marked current for a screen reader. Filed as **QA-091**.
- The collapsed-rail branch in `SettingsNavigation.svelte` (`sidebar.state === 'collapsed'`, the
  one with `tooltipContent`) never rendered for me: collapsing the app sidebar — before or after a
  reload, `data-state="collapsed"`, rail 48px — still leaves the settings nav in its expanded
  276px form with all 14 labels. So the collapsed tooltips are untested, and the nav stays readable
  either way. Not filed: no user-visible defect, just a branch I could not reach.

