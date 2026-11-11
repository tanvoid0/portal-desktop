---
id: QA-115
title: Refresh button spins/disables every ~3s while a pairing QR is on screen
status: verified
severity: polish
kind: ux
area: devices
found: 2026-11-11
mode: tauri
---

## Use case

User has a pairing QR on screen (QA-116's 3s poll running) and looks at the page's
Refresh button.

## Steps

Code read only — `src/routes/devices/+page.svelte:78-79`.

## Expected

Refresh button only shows a busy state for a refresh the user actually triggered.

## Actual

The button's spin/disabled state is bound to `devicesQ.isFetching`, which QA-116's
background poll also flips every ~3 seconds for up to 5 minutes — so the button
flickers busy continuously while a QR card is open, unrelated to anything the user did.

## Evidence

Found during review of QA-116's follow-up fix.

## Repro

Manual — code read, not yet reproduced live.

## Fix

Root cause: the Refresh button's `disabled`/spin state was bound to the shared
`devicesQ.isFetching`, which QA-116's background 3s poll also flips.

Files changed: `src/routes/devices/+page.svelte` — added a local `refreshing` flag set
only inside a new `handleRefreshClick` handler (wraps `refetchDevices`), and the button
now binds to `refreshing` instead of `devicesQ.isFetching`. The background poll still
calls `refetchDevices` directly and no longer touches the button's visual state.

Checks run: `pnpm check`, `pnpm lint` (both clean).

## Verification

2026-11-11, mode: tauri (CDP :9223). `node .claude/qa/scripts/QA-115-verify-postfix.mjs`
— generated a QR to start the 3s background poll, then sampled the Refresh
button's `disabled` attribute every ~100ms for 15s (139 samples) with no user
click: 0 busy samples. Previous (pre-fix) 1s-interval sampling had missed the
flicker; tight sampling now confirms the button stays idle on its own and
only `handleRefreshClick` touches `refreshing`. Fixture revoked after the
run. Verified.
