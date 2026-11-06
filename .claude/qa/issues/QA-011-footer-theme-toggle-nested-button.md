---
id: QA-011
title: Sidebar footer theme toggle is a button inside a button (two tab stops)
status: verified
severity: minor
kind: a11y
area: shell/sidebar-footer
mode: tauri
found: 2026-11-06
---

## Use case

Keyboard / screen-reader user tabbing through the sidebar footer.

## Steps

1. Tab to the sidebar footer; "Toggle theme" takes two stops.

## Expected

One control, one tab stop, one accessible name.

## Actual

`ThemeToggle` renders `<DropdownMenuTrigger><Button>...</Button></DropdownMenuTrigger>`; the trigger is already a `<button>`, so a button is nested in a button (invalid HTML), announced twice. Fix with the trigger's `child` snippet pattern in `src/lib/components/ui/theme-toggle.svelte`. Also the icon only ever shows sun/moon, never that System is selected.

## Evidence

`.claude/qa/shots/QA-011-before-footer-theme-toggle.png`

## Repro

`node .claude/qa/scripts/QA-011.mjs`

## Fix

Root cause: Button nested inside DropdownMenuTrigger (button in button). `theme-toggle.svelte` is app code, not a shadcn primitive (not vendored); used the trigger `child` snippet spreading `props` onto the Button, and icon now follows `currentTheme` so System shows the monitor icon. File: `src/lib/components/ui/theme-toggle.svelte`. Checks: pnpm check 0 errors; eslint + shadcn check clean.

## Verification

2026-11-06, tauri. QA-011.mjs: one button, no nesting, one tab stop; menu Light/Dark/System; footer icon now shows the monitor for System. After-shot: shots/QA-011-after-footer-theme-toggle.png
