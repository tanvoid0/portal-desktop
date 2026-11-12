---
id: QA-150
title: Diagnose & fix / Generate surface a raw "platform POST failed" error when Agent Platform is down; nothing offers Launch or CLI
status: verified
severity: major
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Use case

Owner clicks "Diagnose & fix" on a failed action (or "Generate") with the coder's default
backend (Agent Platform, nothing selected) while the Agent Platform daemon (:18410) is not up.

## Steps

1. Dev build, project aipairs (id 14) -> Actions. Run `Test` (default `pnpm test`, project has no `test` script) -> "Run failed / Pipeline" card.
2. Click "Diagnose & fix".

## Expected

A clear message ("Agent Platform is not running") with the same recovery the Coder page offers (Launch Agent Platform / switch to Claude Code CLI), and a toast. Not counted as a used attempt.

## Actual

Card turns to `Gave up · platform POST failed: error sending request for url (http://127.0.0.1:18410/api/v1/coder/chat/threads)`. No toast, no launch button, no hint. Compare `/ai/coder`, which correctly says "Agent Platform is not running" with Launch/Retry buttons (`sweep-doctor-coder-page-dev.png`). "Gave up" wording also implies the doctor tried and failed, not that no agent ever started.

The failed attempt also leaves an orphan coder thread: `coder_list_threads` shows thread `8d1c362a` (aipairs, system + prompt only, no provider, never answered) created at the failed click. Each retry adds another.

Also on that page: Coder has no provider selected by default (Provider blank, only "Claude Code (CLI)" offered), so doctor passes `provider=undefined` and falls to the platform. Same path in Generate (`runCoderTask`).

## Evidence

`.claude/qa/shots/QA-150-before-diagnose-platform-down.png` (copy of `sweep-doctor-aipairs-diag-final.png`), `sweep-doctor-coder-page-dev.png`.
Code: `src/lib/domains/actions/doctor.ts` `runCoderTask` -> `coderService.createThread` error is passed through unchanged; `state/doctor.svelte.ts` sets message = error text.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-doctor-diagnose.mjs 14 Test QA-150` with no :18410 listener and no provider stored (`localStorage portal.coder.backendProvider` unset).

## Fix

Root cause: runCoderTask called createThread without checking the platform and leaked the raw POST error + orphan thread. Now preflights aiPlatform (non-CLI providers), throws CoderUnavailableError ("Agent Platform is not running."), no thread created, attempt not consumed; send failure deletes the thread. Card says "Could not start" with Launch Agent Platform / Use Claude Code (CLI) buttons + toast; same for Generate. Files: src/lib/domains/actions/{doctor.ts,state/doctor.svelte.ts,components/ProjectActionsPanel.svelte,doctorFlow.test.ts}. Checks: vitest actions (35 pass), eslint 0 errors, prettier. Not run in live app.

Review follow-up: any failure before send succeeds (platform check, createThread, listener registration, send) is a CoderUnavailableError: attempt refunded, empty thread deleted and link cleared, listeners cleaned up; waits for an in-flight platform check.

## Verification

2026-11-12, mode tauri (dev build :9223). Dev provider unset (platform), :18410 down. Staged a failing Typecheck action in aipairs' untracked pipeline.yml. Diagnose -> 'Could not start · Agent Platform is not running.' with Launch Agent Platform and Use Claude Code (CLI) buttons plus a toast; no new coder thread was created (thread list unchanged), no attempt consumed. Shot QA-150-after-platform-down-card.png. Dev provider switched back to claude-cli. Not clicked: the two buttons.
