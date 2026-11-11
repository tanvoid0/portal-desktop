---
id: QA-115
title: Refresh button spins/disables every ~3s while a pairing QR is on screen
status: open
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

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->
Suggested direction (reviewer): bind the busy state to a local `refreshing` flag set only
in the button's own click handler, instead of the shared `isFetching`.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
