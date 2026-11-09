---
id: QA-049
title: Credential master key is silently regenerated when missing, making every stored credential unreadable
status: verified
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

## Decision (owner, 2026-11-09)

Refuse and explain when encrypted rows exist: never silently regenerate the key while
encrypted credentials exist. Error clearly ("master key missing — N credentials can't be
decrypted; restore the key or reset credentials") instead.

Owner action regardless: reconnect GitHub and re-enter any other stored credentials.

## Evidence

QA-045 coder's root-cause notes (its `## Fix`). Nothing stored was modified during QA.

## Repro

manual: code path above.

## Fix

The credentials/vault area was rewired since this was filed (275f618c, "wire vault setup,
unlock and recovery UI to the backend") — the old keychain-based `CredentialService::
get_master_key()` is gone; the bug relocated with it into
`VaultService::read_keychain_master_key()`, called from `setup()` and `unlock_with_keychain()`.
Same silent-regeneration bug, same `NoEntry` branch: no legacy key found → minted a fresh random
key with no check for existing encrypted rows.

Per the owner's decision: `read_keychain_master_key` now takes a `credential_count`; when the
keychain entry and every legacy identifier are gone, it refuses with the new
`CredentialError::MasterKeyMissing(count)` ("master key missing — N credentials can't be
decrypted; restore the key or reset credentials") whenever `credential_count > 0`, instead of
minting a key. Only mints a fresh key when there are zero existing credential rows (genuine
first run). Both call sites (`setup`, `unlock_with_keychain`) now compute and pass the live
credential count.

Files: `src-tauri/portal-core/src/domains/credentials/mod.rs` (new error variant),
`src-tauri/portal-core/src/domains/credentials/services/vault_service.rs`.
Checks: `cargo check` (0 errors) and `cargo test -p portal-core vault_service` (3/3 pass,
including the existing password-reset round-trip test) from `src-tauri/portal-core/`;
`cargo clippy` clean on the touched files.

## Verification

2026-11-09. Could not safely trigger the actual keychain-missing path live — that would mean
deleting the real OS keychain entry backing the owner's actual vault (which has real stored
credentials: 2 GitHub entries, confirmed on `/credentials` this session), which is exactly the
kind of real-data destructive action I'm not permitted to do. Verified by code read + the
project's own test suite instead:

- `src-tauri/portal-core/src/domains/credentials/services/vault_service.rs`:
  `read_keychain_master_key(credential_count: usize)` — confirmed the `NoEntry` branch now
  checks `if credential_count > 0 { return Err(CredentialError::MasterKeyMissing(credential_count)) }`
  before ever minting a fresh key, and both call sites (`setup`, `unlock_with_keychain`) compute
  and pass a live `CredentialEntity::find().all(&self.db).await?.len()` before calling it —
  matches the `## Fix` description exactly.
- `cargo test -p portal-core vault_service` from `src-tauri/portal-core/`: 3/3 pass
  (`recovery_key_round_trips_through_formatting_and_pasted_whitespace`,
  `hkdf_and_pbkdf2_wrap_unwrap_round_trip`,
  `password_reset_rewraps_mk_and_keeps_existing_credentials_decryptable`).

Verified (code read + tests; live trigger not safely possible on this real vault).
