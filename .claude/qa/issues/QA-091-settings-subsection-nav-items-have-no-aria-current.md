---
id: QA-091
title: Settings sub-section nav items never get aria-current, so five sections have no current page
status: open
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

<!-- coder -->

## Verification

<!-- qa-tester -->
