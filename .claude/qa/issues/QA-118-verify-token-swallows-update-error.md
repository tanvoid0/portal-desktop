---
id: QA-118
title: pairing_service verify_token silently swallows the first-use update error
status: open
severity: minor
kind: bug
area: devices/pairing
found: 2026-11-11
mode: tauri
---

## Use case

Phone scans a QR pairing code; backend clears `expires_at`/updates `last_used_at` on
first use.

## Steps

Code read only — `src-tauri/portal-core/src/domains/network/services/pairing_service.rs:415`.

## Expected

If the DB update on first use fails, it should be logged/surfaced somehow.

## Actual

`let _ = device_approval::Entity::update(m)` discards the error. If it fails, the grant
never flips to "used" — `last_used_at`/`expires_at` stay as before, every retry re-attempts
the same update, and if every retry fails until the window closes the phone is silently
locked out with no error anywhere.

## Evidence

Found during review of QA-116's fix.

## Repro

Manual — code read, not yet reproduced live.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
