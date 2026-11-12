---
id: QA-153
title: Diagnose & fix re-runs the command the agent just wrote into .portal/pipeline.yml automatically, before the owner has seen it
status: verified
severity: major
kind: ux
area: actions/doctor
mode: tauri
found: 2026-11-12
---

## Use case

Owner trusts the Coder approval gate so that agent-authored changes do nothing until reviewed.

## Steps

1. Devstrail (id 4) -> Run `Test` -> Diagnose & fix. Agent writes `.portal/pipeline.yml` (`test: run: pnpm typecheck`).
2. Do not review or approve anything in Coder.

## Expected / Actual

Portal reloads the file and immediately runs the agent's command (run history: `pnpm --ignore-workspace typecheck`, 16:50:05) without showing it to the owner first. Harmless here, but `run:` is arbitrary shell chosen by the agent; the gate is "review after write" and shell approvals belong to the agent's own tool calls, not to Portal's verify step. A confused or hostile agent could write a destructive or deploy command and Portal would execute it on the owner's behalf.

## Why needs-decision

By design for the closed loop (verify green). Options: show the command and confirm (`confirmAction`) before the first verify run; or auto-verify only test/lint/build/typecheck-looking commands.

## Evidence

Run-history row right after the agent's write; `.claude/qa/shots/sweep-doctor-devstrail-amber-report-dev.png`.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-diag-full.mjs 4 Test QA-153`

## Fix

Per owner decision: after the agent turn, if the resolved action command differs from the original, status awaiting-verify shows Was / Agent wrote (not run yet) and nothing runs until the Verify button (verifyProposal). Unchanged command still auto-verifies. Covered by doctorFlow.test.ts. Files: src/lib/domains/actions/{doctor.ts,state/doctor.svelte.ts,components/ProjectActionsPanel.svelte,doctorFlow.test.ts}. Checks: vitest actions (35 pass), eslint 0 errors, prettier. Not run in live app.

Review follow-up: gate is now fail-closed: it compares command + package.json scripts + workflow shadowing against the pre-fix snapshot; a missing/shadowed action, changed script, unreadable catalog or no baseline never auto-runs (unreadable catalog gives up). Verify runs only the action (run([id])), only if disk still matches what was shown, else re-shows. Proposal card is rendered from doctor state (survives remount) and shows changed scripts. Tests: doctorFlow.test.ts.

## Verification

2026-11-12, mode tauri (dev build :9223). Agent changed the staged action from 'pnpm run typecheck-all' to 'npm run typecheck'. Panel showed 'Review the agent's change · Nothing has run · Was / Agent wrote (not run yet) · Verify'; run history had no new row until I clicked Verify (then 'npm run typecheck' ran, exit 0). Generate disabled meanwhile. Shot QA-153-after-awaiting-verify-card.png.
