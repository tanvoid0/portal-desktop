---
id: QA-113
title: Expired, never-scanned QR grant rows pile up in the database forever
status: open
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

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
