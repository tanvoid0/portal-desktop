# QA board

The channel between `qa-tester` (finds, verifies, closes) and `coder` (fixes).
One file per issue in `issues/`. No index — the files are the index:

```bash
grep -l "^status: open" .claude/qa/issues/*.md      # what needs fixing
grep -l "^status: fixed" .claude/qa/issues/*.md     # what needs verifying
```

## Layout

| path                          | what                                                      |
| ----------------------------- | --------------------------------------------------------- |
| `issues/QA-NNN-short-slug.md` | one issue, template below                                 |
| `scripts/app.mjs`             | helper: open a route, screenshot, collect console errors  |
| `scripts/QA-NNN.mjs`          | repro for an issue — the coder reruns it to prove the fix |
| `shots/`                      | screenshots (gitignored)                                  |

## Status flow

```
open ──► fixing ──► fixed ──► verified        (closed)
  ▲                   │
  └──── reopened ◄────┘
any ──► needs-decision                         (product/design call → lead asks the owner)
```

| status           | set by                           | means                                                 |
| ---------------- | -------------------------------- | ----------------------------------------------------- |
| `open`           | qa-tester                        | filed, nobody on it                                   |
| `fixing`         | coder                            | picked up                                             |
| `fixed`          | coder                            | fix landed, `## Fix` filled in                        |
| `verified`       | qa-tester                        | re-tested, gone. Only the tester closes.              |
| `reopened`       | qa-tester                        | still broken — `## Verification` says what            |
| `needs-decision` | either                           | not a bug call — needs the owner. Reason in the body. |
| `wontfix`        | lead only, on the owner's say-so |                                                       |

**Ownership:** the tester writes everything except `## Fix`; the coder writes only
`status:` and `## Fix`. Nobody deletes issue files.

## Severity

|           |                                                                              |
| --------- | ---------------------------------------------------------------------------- |
| `blocker` | use case can't be completed, crash, data loss or corruption                  |
| `major`   | wrong result, misleading UI, no sane workaround, keyboard user stuck         |
| `minor`   | works but rough: no feedback, inconsistent component, poor empty/error state |
| `polish`  | visual nit: spacing, alignment, copy                                         |

## Issue template

```markdown
---
id: QA-007
title: Relink dialog accepts a path that does not exist
status: open
severity: major
kind: bug # bug | ux | a11y | perf | copy
area: projects/audit # route or domain
mode: tauri # tauri (real backend) | browser (no backend)
found: 2026-11-05
---

## Use case

What the user is trying to do.

## Steps

1. …

## Expected

## Actual

## Evidence

`.claude/qa/shots/QA-007-relink.png`, console errors, invoke errors.

## Repro

`node .claude/qa/scripts/QA-007.mjs` (or "manual — steps above")

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
```

## Running the app for testing

- **Real app (full backend):** start with CDP on so scripts can drive the WebView2:
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 pnpm tauri:dev`
  (PowerShell: `$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9222'; pnpm tauri:dev`)
- **Browser mode:** the vite server on `:1420` in Chromium. Every `invoke` returns an
  empty value — good for layout, navigation, empty states and a11y; useless for data flows.

`app.mjs` picks the real app when `:9222` answers, else browser mode, and prints which.

- In browser mode, console errors mentioning `transformCallback` or `reading 'invoke'`
  are the missing Tauri bridge — noise, not bugs.
- The app scrolls inside its main pane, so `fullPage` screenshots stop at the window.
  Scroll the pane or screenshot a locator for content below the fold.
