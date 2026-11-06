---
id: QA-064
title: Deleting a credential only removes it from the local list; it comes back after reload while a "Credential deleted" toast is shown
status: verified
severity: blocker
kind: bug
area: credentials
mode: tauri
found: 2026-11-06
---

## Use case

Delete a credential I no longer want.

## Steps

1. `/credentials`, Add Credential `QA credential` (fake value).
2. Delete it, confirm "Delete".
3. Reload the page.

## Expected

Credential is deleted in the backend and stays gone.

## Actual

Card disappears and "Credential deleted" toast is raised, but after reload the credential is back (including its secret in the DB).
`CredentialVault.svelte` `handleDeleteCredential` only calls `credentialActions.removeCredential(id)` (store filter) and never calls
`credentialService.deleteCredential` / `delete_credential`. The backend command works: invoking `delete_credential` directly removed my fixture.
Same pattern for create/update in `handleFormSave` is worth a look (the form saves itself, the vault only updates the store).

## Evidence

`.claude/qa/shots/QA-064-before-credential-back-after-reload.png`; script prints "still present after reload (BUG if 1): 1".

## Repro

`node .claude/qa/scripts/QA-064.mjs` (cleans its own fixture via invoke).

## Fix

Root cause: `CredentialVault.handleDeleteCredential` only removed the item from the local store. It now awaits `credentialService.deleteCredential` (camelCase args), then updates the store; on failure it toasts the error and keeps the item.
File: credentials/components/CredentialVault.svelte. Checks: eslint 0 errors, prettier clean, pnpm check clean for credentials/ (unrelated errors elsewhere). Not run live.

## Verification

<!-- qa-tester -->

2026-11-06 (last round), tauri. Created QA credential, renamed it, reloaded (persisted), deleted via confirm dialog, reloaded: gone. shots/QA-064-after-credential-gone.png.
