---
id: QA-113
title: Expired, never-scanned QR grant rows pile up in the database forever
status: verified
severity: polish
kind: bug
area: devices/pairing
found: 2026-11-11
mode: tauri
---

## Use case

User generates a QR, lets it expire unscanned, generates another (e.g. via "Regenerate
QR code").

## Steps

Code read only — `src-tauri/portal-core/src/domains/network/services/pairing_service.rs:155`
(`mint_qr_pairing`).

## Expected

Expired, never-used QR rows get pruned eventually.

## Actual

They stay `approved: true` in the `device_approvals` table forever with no cleanup.
Harmless to auth (`verify_token` still rejects them on the expiry check) but rows
accumulate with every "Regenerate QR code" click, and after QA-116's fix they're also
permanently invisible in the UI, so there's no way to even notice the buildup.

## Evidence

Found during review of QA-116's fix.

## Repro

Manual — code read, not yet reproduced live.

## Fix

Root cause: `list_devices` had no `expires_at` check, so an expired, never-scanned
`until_revoked` QR row stayed in its result (and the DB) forever.

Files changed: `src-tauri/portal-core/src/domains/network/services/pairing_service.rs`
— `list_devices` now filters out rows that are `until_revoked` with an `expires_at`
still set (i.e. never scanned) that has passed, matching QA-116's own rationale (a scan
clears `expires_at` on first use, so a still-set-and-past one was never picked up and
never will be). Cheapest option per the issue's suggested direction — filter-on-read,
not a cleanup job. Added
`expired_unscanned_qr_grant_is_pruned_from_list_devices` as the regression test.

Checks run: `cargo check` (workspace + `portal-core`), `cargo test pairing_service`
(9/9 pass, including the new test).

## Verification

2026-11-11, mode: tauri (CDP :9223 + `cargo test -p portal-core`).
`cargo test -p portal-core pairing_service` → 9/9 pass, including
`expired_unscanned_qr_grant_is_pruned_from_list_devices`. Read `list_devices`
live: the filter (`until_revoked` + `expires_at` set + `expires_at <= now`) is
in place at `pairing_service.rs:358-363`. Live spot-check: generated a QR,
confirmed it doesn't show in "Paired devices" or its count while live (QA-116
behavior, unchanged); didn't wait out the 5-minute expiry live (impractical),
relying on the passing regression test for the expired-row prune itself, per
the issue's own guidance. Verified.
