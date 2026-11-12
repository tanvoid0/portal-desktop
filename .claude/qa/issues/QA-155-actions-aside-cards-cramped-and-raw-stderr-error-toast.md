---
id: QA-155
title: Actions side column cramps the doctor cards; failed-run toast dumps raw stderr and lingers
status: verified
severity: polish
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Steps

1. Devstrail -> Actions -> run a failing action; let Diagnose finish (1280x800).

## Actual

- "Portal default needs a fix" title wraps over three lines with its badge beside it; the "Generate a pipeline" description wraps to ~7 lines in the same ~215 px column; code blocks are tight.
- The error toast duplicates the failure card with raw output (`Exit code 1 ERR_PNPM_NO_SCRIPT Missing script: test Command "test" not found…`, newlines collapsed) and was still on screen ~2 min after the run.

## Evidence

`.claude/qa/shots/QA-155-before-narrow-aside-and-stale-toast.png`

## Repro

manual — steps above.

## Fix

Layout: row layout/aside width moved from lg to xl (aside xl:w-80) so the main column is wider; report card header wraps. Toast now short ("<action> failed" + exit code / first error line, 5s) instead of raw stderr. Visual check not done. Files: src/lib/domains/actions/{doctor.ts,state/doctor.svelte.ts,components/ProjectActionsPanel.svelte,doctorFlow.test.ts}. Checks: vitest actions (35 pass), eslint 0 errors, prettier. Not run in live app.

## Verification

2026-11-12, mode tauri (dev build :9223). Failure toast is now short ('typecheck failed Exit code 1') and gone within ~6 s. Cards readable at 1280 px, though the three-column layout is still tight (actions list ~310 px) and the 'Fixed · pending' card has no heading; left as-is. Shot QA-152-after-fixed-pending-review-card.png.
