---
id: UX-005
title: Settings nav is 993px tall for 14 sections, and every page states its title twice
status: fixed
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

<!-- qa-tester -->
