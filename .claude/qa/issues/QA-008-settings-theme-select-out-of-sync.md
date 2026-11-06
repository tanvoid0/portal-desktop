---
id: QA-008
title: Settings > General "Theme" select ignores the real theme and does nothing when changed
status: verified
severity: major
kind: bug
area: settings/general
mode: tauri
found: 2026-11-06
---

## Use case

Check or change the theme from Settings > General > Appearance.

## Steps

1. Header theme menu -> Dark (app turns dark, survives reload).
2. Open Settings > General.
3. Look at the Theme select; then pick Light/Dark in it.

## Expected

Select shows Dark (the active preference). Picking a value applies it (live, or on Save).

## Actual

Select always shows "System" (value comes from saved `settings.app.theme`, a second source of truth separate from `themeStore`). Picking Dark in it changes nothing visible and does not enable Save. The header chip says `Theme: system` while the app is dark. Two controls for one setting that disagree. (Product call: maybe drop the select and keep the header toggle; coder/lead decide which wins.)

## Evidence

`.claude/qa/shots/QA-008-before-theme-select-out-of-sync.png`. Code: `src/lib/domains/settings/components/GeneralSettings.svelte` L136-142 (`onUpdate({theme})` only writes the settings store).

## Repro

`node .claude/qa/scripts/QA-008.mjs` (restores theme to System at the end).

## Fix

Per owner decision: removed the Theme select (GeneralSettings.svelte) and the `Theme:` header chip (src/routes/settings/+layout.svelte). Header/footer toggle is the only control; no sync.
Checks: pnpm check 0 errors; eslint 0 errors; prettier clean on touched files. Not run in-app.

## Verification

2026-11-06, tauri. Per owner decision the Theme select and the Theme chip are gone: General shows only Language / Time Format / update-channel selects. Header toggle is the sole control (QA-006 re-checked, still fine). After-shot: shots/QA-008-after-no-theme-select.png

## Decision

2026-11-06, owner (via lead): keep the header/footer theme toggle as the only control.
Remove the Theme select from Settings > General (and the `Theme:` header chip if it reads
`settings.app.theme`). Don't migrate or sync `settings.app.theme` into `themeStore`.
