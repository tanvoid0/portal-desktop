---
id: QA-118
title: pairing_service verify_token silently swallows the first-use update error
status: verified
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

Root cause: `verify_token` discarded the first-use update's `Result` with `let _ = ...`
instead of propagating it, the only spot in the file that didn't follow this service's
own convention (every sibling mutation — `approve`, `revoke`, `reject`,
`update_capabilities`, `rename_device` — propagates the DB error via
`.map_err(...)?`).

Files changed: `src-tauri/portal-core/src/domains/network/services/pairing_service.rs`
— `verify_token`'s final update now uses the same `.map_err(|e| format!("Failed to
update device approval: {e}"))?` pattern as its siblings, so a failed write now fails
the call (and the `/control` guard, which already treats an `Err` the same as an
unmatched token) instead of silently leaving the grant in its pre-use state.

Checks run: `cargo check` (workspace + `portal-core`), `cargo test pairing_service`
(9/9 pass).

## Verification

2026-11-11, mode: tauri (CDP :9223 + `cargo test -p portal-core`), real phone.
Confirmed the code fix: `verify_token`'s final `device_approval::Entity::update(m)`
now ends `.map_err(|e| format!("Failed to update device approval: {e}"))?`
(pairing_service.rs:425-428), matching the sibling-mutation pattern described
in the Fix. `cargo test -p portal-core pairing_service` → 9/9 pass (no
dedicated QA-118 test — confirmed none of the 9 test names target this path,
consistent with the issue noting the failure-injection path is hard to
black-box). Happy-path regression: real-phone scan → verify → paired covered
live twice (via QA-117's `QA-117-scan-autoclear.mjs` and separately during
QA-116-style checks), both picked up within ~2s with `last_used_at` set and
`expires_at` cleared, so the normal (non-error) path through this exact code
still works end to end. The failure-injection path itself remains
code-review-verified only, as expected. Verified.
