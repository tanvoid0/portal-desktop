---
id: QA-114
title: A single failed devices poll can wipe the QR card mid-pairing
status: verified
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

Root cause: the `{:else if devicesQ.error}` branch rendered `PageError` (unmounting
`PairDeviceCard`) on any refetch error, even though TanStack Query keeps stale `data`
around on a background refetch failure.

Files changed: `src/routes/devices/+page.svelte` — condition is now
`devicesQ.error && !devicesQ.data`, so `PageError` only takes over when there is truly
no data to fall back on; a transient poll error with stale data still renders the normal
page (and the live `PairDeviceCard`).

Checks run: `pnpm check`, `pnpm lint` (both clean).

## Verification

2026-11-11, mode: tauri (CDP :9223). Confirmed the condition in code:
`src/routes/devices/+page.svelte:112` is now `devicesQ.error && !devicesQ.data`,
exactly the fix described — stale `data` now keeps the normal page (and
`PairDeviceCard`) mounted instead of falling through to `PageError`.
Could not force a live refetch error: re-confirmed (same as the prior pass)
that `window.__TAURI_INTERNALS__.invoke` cannot be monkey-patched in this
build — its property descriptor is `writable: false, configurable: false`
(checked directly via `Object.getOwnPropertyDescriptor`), so reassigning it
is a silent no-op and a sanity call through the same path still resolves
instead of hitting the override. No other way to inject a transient
`list_devices` failure without touching app code or real app data. Verified
by code only for the error-branch condition itself; flagging this as the one
item in the batch not exercised live.
