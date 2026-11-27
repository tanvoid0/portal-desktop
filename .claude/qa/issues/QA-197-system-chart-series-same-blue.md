---
id: QA-197
title: System page charts draw both series in the same blue (CPU/Memory, Down/Up)
status: fixed
severity: major
kind: ux
area: system
mode: tauri
found: 2026-11-27
---

## Use case

Read the "Last 60 seconds" and "Network" charts: which line is CPU and which is memory,
which is download and which is upload.

## Steps

1. Open /system and let the charts fill.
2. Compare the two lines and the two legend dots, light and dark theme.

## Expected

Two clearly different colours per chart.

## Actual

Dark: `rgb(59,130,246)` vs `rgb(60,131,246)` — identical to the eye. Light:
`rgb(37,99,235)` vs `rgb(60,131,246)` — two near-identical blues. When the lines cross they
cannot be told apart, and the legend does not help. Uncommitted change under test.
Cause: `bg-primary`/`text-primary` and `bg-status-info`/`text-status-info` are the same hue
(`src/app.css` --primary vs --status-info), used in `src/routes/system/+page.svelte` (legend
snippet) and `src/lib/domains/disk/components/Trace.svelte`.

## Evidence

`.claude/qa/shots/QA-197-before-chart-series.png`,
`.claude/qa/shots/sweep-system-chart-full-window-dark.png` (lines cross mid-chart).

## Repro

`node .claude/qa/scripts/QA-197.mjs`

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

Root cause: `--primary` and `--status-info` are the same hue. Series B now uses `status-warning` (amber) against `primary` (blue).
Files: `src/lib/domains/disk/components/Trace.svelte`, `src/routes/system/+page.svelte`.
Checks: `pnpm check` 0 errors, disk vitest 9/9. Repro script not rerun (needs the running app); fix judged from the token values.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
