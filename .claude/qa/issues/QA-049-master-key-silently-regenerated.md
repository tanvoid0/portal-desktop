---
id: QA-049
title: Credential master key is silently regenerated when missing, making every stored credential unreadable
status: needs-decision
severity: major
kind: bug
area: credentials
mode: tauri
found: 2026-11-06
---

## Use case

The owner stores credentials (GitHub token, API keys) and expects them to stay readable.

## Steps

Found while root-causing QA-045 (code reading plus the live decrypt failure):
1. The 32-byte AES-GCM master key lives in the OS keychain, service `com.tanvoid0.portal-desktop`,
   entry `credential-master-key-v1` (`get_master_key()` in `credential_service.rs`).
2. If that entry is missing (keychain reset, app identifier change, entry deleted), `get_master_key()`
   checks `LEGACY_APP_IDENTIFIERS`, then silently generates a fresh random key.
3. Every credential sealed under the old key now fails with `ring::error::Unspecified`.

## Expected

Never silently replace the key while encrypted rows exist. Either refuse with a clear error
("master key missing — N credentials can't be decrypted; restore the key or reset credentials"),
or only generate a new key when the credentials table is empty.

## Actual

New key generated without warning; the owner's GitHub token (and probably all other stored
credentials) are now permanently unreadable. Surfaced only as GitHub calls failing (QA-045).

## Decision needed

Product/security call for the owner: (a) refuse and explain when encrypted rows exist
(recommended), (b) generate only when the table is empty, (c) keep current behaviour. Also: what
the recovery path in the UI should be (re-enter credentials, bulk reset).

Owner action regardless: reconnect GitHub and re-enter any other stored credentials.

## Evidence

QA-045 coder's root-cause notes (its `## Fix`). Nothing stored was modified during QA.

## Repro

manual: code path above.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
