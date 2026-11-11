---
id: QA-117
title: Unscanned QR grant can't be revoked during its 5-minute window (hidden from every list)
status: verified
severity: minor
kind: bug
area: devices
found: 2026-11-11
mode: tauri
---

## Use case

User generates a pairing QR, then wants to cancel it before it's scanned (e.g. wrong
phone, second thoughts, worried someone else saw the code).

## Steps

1. Devices → Pair a phone → Generate QR code.
2. Before scanning (or letting it expire), try to find a way to cancel/revoke it.

## Expected

Some way to cancel a live, unscanned QR grant.

## Actual

QA-116's fix (excluding `until_revoked` rows with `expires_at` set from every list)
correctly stops it showing as falsely "Paired", but as a side effect it's now invisible
everywhere — not in "Paired devices", not in "Pending requests", not in history — so
there is no `DeviceRow`/revoke action reachable for it at all during its window.
"Regenerate QR code" also mints a new grant without revoking the old one, so an unscanned
previous grant stays live and unrevocable until it naturally expires.

## Evidence

Found during review of QA-116's fix (`src/routes/devices/+page.svelte`).

## Repro

Manual — steps above.

## Fix

Root cause: QA-116's fix correctly hid an unscanned `until_revoked` QR grant from every
list, but left no UI affordance to cancel it during its 5-minute live window.

Files changed:
- `src/lib/domains/devices/components/PairDeviceCard.svelte` — `generate()` now keeps
  the minted `device_id` in state; a new "Cancel" button (shown next to "Regenerate QR
  code" while a QR is on screen) calls the existing `revokeDevice` on it. A new
  `pairedDeviceIds` prop + `$effect` auto-clears the card once the grant actually shows
  up in "Paired devices" (i.e. it was scanned), so Cancel can never fire against an
  already-real paired device.
- `src/routes/devices/+page.svelte` — passes a new `pairedDeviceIds` derived `Set` of
  paired device_ids into `PairDeviceCard`.

Checks run: `pnpm check`, `pnpm lint` (both clean).

## Verification

2026-11-11, mode: tauri (CDP :9223), real phone (`adb-R5GL150LRCK-N75UEH._adb-tls-connect._tcp`,
"Tanveer's S26 Ultra").

- `node .claude/qa/scripts/QA-117-cancel-flow.mjs`: generated a QR (Cancel
  button appears next to "Regenerate QR code"), clicked Cancel before
  scanning — card resets to "Generate QR code" with the QR image gone, and
  the grant shows up in History as `Revoked`. Screenshots:
  `QA-117-before-cancel-click.png`, `QA-117-after-cancel-history.png`.
- `node .claude/qa/scripts/QA-117-scan-autoclear.mjs`: generated a QR, fired
  the bearer token from the real phone over adb (phone-side `curl` to the LAN
  pairing endpoint, same technique as QA-116's e2e check) — the grant picked
  up as paired and the card auto-cleared to "Generate QR code" (no stale
  Cancel button left on the now-real paired device) within ~2s, no manual
  refresh. Screenshot: `QA-117-after-scan-autoclear.png`.

One hiccup along the way: two earlier throwaway probe scripts of mine hung
because my own Playwright locator regex was wrong (`/^Cancel$/` doesn't match
via `hasText` in this Playwright version — plain-string `hasText: 'Cancel'`
does), not an app bug; left two orphaned unrevoked QR fixtures from those
failed runs, found via raw `list_devices` and revoked directly. All fixtures
(cancel-flow's row, the scanned/paired row, the two orphans) confirmed
revoked; final live-row list matches the pre-test baseline (Pending 2, Paired
1, same 3 pre-existing "Browser" rows, no leftover "Unnamed device" rows).
Verified.
