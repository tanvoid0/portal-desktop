---
id: QA-114
title: A single failed devices poll can wipe the QR card mid-pairing
status: open
severity: minor
kind: bug
area: devices
found: 2026-11-11
mode: tauri
---

## Use case

User has a pairing QR on screen (QA-116's 3s poll is running) and a transient backend
error hits one poll.

## Steps

Code read only — `src/routes/devices/+page.svelte:87`.

## Expected

A brief refetch error shouldn't tear down UI the user is actively using.

## Actual

On a refetch error TanStack Query sets `devicesQ.error` even though it keeps the old
`data`. The page checks `error` first and renders `PageError`, and `PairDeviceCard` lives
inside the `{:else}` branch — so it unmounts and its `qrDataUrl` state (the whole QR
card) is lost. This could already happen on any refetch, but QA-116's 3s/5min poll makes
hitting a brief backend hiccup while pairing much more likely.

## Evidence

Found during review of QA-116's follow-up fix.

## Repro

Manual — code read, not yet reproduced live.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->
Suggested direction (reviewer): only render `PageError` when there's no data
(`devicesQ.error && !devicesQ.data`), or move `PairDeviceCard` outside the loading/error
branch.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
