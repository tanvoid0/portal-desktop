---
id: QA-091
title: Settings sub-section nav items never get aria-current, so five sections have no current page
status: verified
severity: minor
kind: a11y
area: settings
mode: tauri
found: 2026-11-07
---

## Use case

A screen-reader user opens Settings and wants to know which section they are on.

## Steps

1. `/settings/ai/providers`.
2. Query `document.querySelectorAll('[aria-current]')`.
3. Repeat on `/settings/languages`, `/settings/package-managers`, `/settings/frameworks`,
   `/settings/ai/training`.

## Expected

The active nav item is marked `aria-current="page"`, the way the top-level items already are.

## Actual

Nothing on the page carries `aria-current`:

```
/settings/ides            aria-current=["IDEs:page"]
/settings/autonomy        aria-current=["Autonomy:page"]
/settings/ai/providers    aria-current=[]
/settings/languages       aria-current=[]
```

Five of the fourteen sections — Languages, Package Managers, Frameworks, Providers, Training Data —
are sub-section items, and the sub-section branch of
[SettingsNavigation.svelte:182-193](../../../src/lib/domains/settings/components/SettingsNavigation.svelte#L182)
sets only the `bg-accent` class, not `aria-current`, while the top-level branch at line 201 does set
it. The active state is visible but not announced.

Same block also styles sub-items `h-auto px-3 py-2` (38px) against the top-level `h-8` (32px), so
the rail has two row heights; worth aligning in the same edit.

## Evidence

`node .claude/qa/scripts/ux005-verify2.mjs` — output quoted above.
`.claude/qa/shots/UX-005-after-settings-general-1280.png`

## Repro

`node .claude/qa/scripts/ux005-verify2.mjs`

## Fix

Root cause: the sub-section `Button` branch in `SettingsNavigation.svelte` only set the
`bg-accent` class for the active state, never `aria-current`, unlike the top-level branch.
Added `aria-current={isActive(subSection.path) ? 'page' : undefined}` and, in the same edit,
changed `h-auto px-3 py-2` to `h-8` so sub-items match the top-level row height (38px → 32px).

File: `src/lib/domains/settings/components/SettingsNavigation.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-091.mjs` against
`/settings/ai/providers`, `/settings/languages`, `/settings/package-managers`,
`/settings/frameworks`, `/settings/ai/training` (the five sub-section routes from the
original repro) plus `/settings/ides` and `/settings/autonomy` as regression checks on the
top-level branch. All seven now report `aria-current="page"` on the active nav item. Row
heights visually consistent in `.claude/qa/shots/QA-091-after-settings-nav.png`. Verified.
