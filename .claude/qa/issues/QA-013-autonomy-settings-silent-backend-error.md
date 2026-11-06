---
id: QA-013
title: Autonomy settings page hides backend failure and shows defaults as real state
status: verified
severity: major
kind: bug
area: settings/autonomy
mode: tauri
found: 2026-11-06
---

## Use case

See and change whether autonomous actions are enabled.

## Steps

1. Open `/settings/autonomy` in the real app.

## Expected

Real state, or a visible error/unavailable state (`PageError` with retry); never a toggle that looks authoritative.

## Actual

Backend calls fail with `Autonomy service not found` (get enabled state, get level, approval stats), but the page renders "Autonomy Enabled: Disabled" and level "Balanced" with no error and no toast. The owner cannot tell these are defaults. Also the logger mangles the string error into a char-indexed object (`{"0":"A","1":"u",...}`): `log.error(msg, error)` spreads a string; pass `{ error: String(e) }` or normalise in the logger.

## Evidence

`.claude/qa/shots/QA-013-before-autonomy-silent-error.png`, console `[ERROR][AutonomyService] Failed to get autonomy enabled state`.

## Repro

`node .claude/qa/scripts/QA-013.mjs`

## Fix

Frontend done: autonomyService.getLevel/getEnabled no longer swallow errors into defaults (rethrow); AutonomySettings shows PageError 'Autonomy settings unavailable' with Retry instead of defaults/toast; logger createScoped no longer spreads strings/Errors (wraps as {error, context}).
Files: src/lib/domains/autonomy/services/autonomyService.ts, src/lib/domains/settings/components/AutonomySettings.svelte, src/lib/domains/shared/services/logger.ts.
Review follow-ups: Enable switch handler now takes the new value (it previously sent the old value, wrong toast, wrong revert); Retry keeps the error card until a load succeeds; approval-stats failure shows its own error + Retry; single load() for mount and Retry.
REMAINING (Rust, not edited): src-tauri/src/domains/autonomy/commands.rs. The global map AUTONOMY_SERVICES (L14) is only populated with the "default" service lazily inside evaluate_autonomous_action (L46-48). get_autonomy_level (L110), set_autonomy_level (L134), get_autonomy_enabled (L146), set_autonomy_enabled (L156), get_approval_stats (L168) use `.get("default")?` and return 'Autonomy service not found' until an action has been evaluated. Fix: seed "default" in get_services_map's get_or_init (or use entry().or_insert_with) in all commands. Until rebuilt the page will show the error state.
pnpm check: 0 errors; eslint 0 errors; prettier clean on touched files (remaining prettier warnings: CLAUDE.md, projects/generated/UnavailableProject.ts, pre-existing); shadcn check passed. Not run in-app (tester verifies).

Rust part (lead, 2026-11-06): `src-tauri/src/domains/autonomy/commands.rs` now seeds the
`"default"` service in `get_services_map()`'s `get_or_init`; the lazy insert in
`evaluate_autonomous_action` is gone. Checked in tauri mode after the rebuild:
`get_autonomy_enabled` → true, `get_autonomy_level` → "Balanced", `get_approval_stats` → {};
`/settings/autonomy` renders with no error card, no console errors
(`shots/QA-013-after-autonomy-loads.png`). Known limit, not fixed here: the service is
in-memory only, so level/enabled reset to the defaults (enabled, Balanced) on every restart.

## Verification

2026-11-06, tauri after the Rust rebuild. /settings/autonomy loads real state (Active, Balanced), no error card, no console errors. Not tested: toggling (state is in-memory, would change owner config). After-shot: shots/QA-013-after-autonomy-ui.png
