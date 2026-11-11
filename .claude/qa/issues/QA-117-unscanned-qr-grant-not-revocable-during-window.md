---
id: QA-117
title: Unscanned QR grant can't be revoked during its 5-minute window (hidden from every list)
status: open
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

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
