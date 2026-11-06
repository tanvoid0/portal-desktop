---
id: QA-014
title: Settings a11y: "Enable Agent Platform" switch has no name; main nested in main
status: verified
severity: minor
kind: a11y
area: settings
mode: tauri
found: 2026-11-06
---

## Use case

Screen reader / keyboard use of Settings.

## Steps

1. Open `/settings/ai`: the "Enable Agent Platform" switch has no `aria-label`, `aria-labelledby` or associated label.
2. On any `/settings/*` page: `document.querySelectorAll('main main').length` is 1 (settings layout `<main>` inside the app shell `<main>`).

## Expected

Switch announced as "Enable Agent Platform"; a single `main` landmark.

## Actual

Unnamed switch; two nested `main` landmarks.

## Evidence

(Before-shot deleted 2026-11-06 by the lead on the owner's say-so: it showed the agent-platform API token in plain text. See QA-016.)

## Repro

`node .claude/qa/scripts/QA-014.mjs`

## Fix

Root cause: Label had no `for`/Switch no id; ShellSidebarLayout rendered its own `<main>` inside the app-shell `<main>`. Label for=agent-platform-enabled + Switch id; shell `<main>` -> `<div>` (also fixes cloud/sdk/utilities).
Files: src/lib/domains/ai/components/providers/AIProviderSettings.svelte, src/lib/components/shell/shell-sidebar-layout.svelte.
pnpm check: 0 errors; eslint 0 errors; prettier clean on touched files (remaining prettier warnings: CLAUDE.md, projects/generated/UnavailableProject.ts, pre-existing); shadcn check passed. Not run in-app (tester verifies).

## Verification

2026-11-06, tauri. QA-014.mjs: 0 unnamed switches, 0 nested main. /sdk /cloud/workloads /utilities render with one main, no horizontal overflow at 1024x600 (shots/sweep-sdk-min-1024.png etc). After-shot: shots/QA-014-after-ai-switch-named.png
