---
id: QA-043
title: Credentials page is a bare "500 Internal Error" (TypeError in CredentialCard) - raw snake_case rows never normalized
status: verified
severity: blocker
kind: bug
area: credentials
mode: tauri
found: 2026-11-06
---

## Use case

Open Credentials, see, add, edit and delete stored credentials.

## Steps

1. Open `/credentials` with at least one stored credential.

## Expected

Header, add button and a card per credential (masked).

## Actual

The main pane shows only "500 Internal Error". Console:
`TypeError: Cannot read properties of undefined (reading 'replace')` at `CredentialCard.svelte` (`credential.type.replace('_', ' ')`, line 153). Because the page dies, Add/Edit/Delete could not be reached or tested at all.

Likely root cause: `credentialService.getCredentials()` returns `invoke<Credential[]>('get_credentials')` raw. The Rust `Model` (`src-tauri/portal-core/src/domains/credentials/entities/mod.rs`) serializes `credential_type`, `encrypted_value`, `last_used`, `created_at`, and `tags` as a JSON *string*, while the TS `Credential` expects `type`, `encryptedValue`, `lastUsed`, `createdAt`, `tags: string[]`. There is no normalizer (CLAUDE.md data-flow step 2), so `credential.type` is undefined, and `tags.length` / `#each tags` operate on a string. Also no error boundary: one bad row takes down the whole route.

## Evidence

`.claude/qa/shots/QA-043-before-credentials-500.png` (no values shown), console error above.

## Repro

`node .claude/qa/scripts/QA-043.mjs`

## Fix

Root cause: `credentialService` returned raw Rust rows (`credential_type`, `encrypted_value`, JSON-string `tags`/`metadata`) with no normalizer, so `credential.type` was undefined.
Fix: new `credentials/utils/normalizeCredential.ts` (defensive: missing type -> `other`, bad tags/metadata JSON -> empty, bad dates -> undefined); used by create/get/update/list/search in `credentialService.ts` (also covers `variableService.getCredential`).
Files: src/lib/domains/credentials/utils/normalizeCredential.ts (new), src/lib/domains/credentials/services/credentialService.ts. Also fixed create's log line that referenced an undefined id.
Checks: pnpm check 0 errors; eslint 0 errors; prettier clean. Dates: naive UTC strings from Rust get a `Z` appended before parsing so expiry isn't shifted by the local offset. Not rerun against the live app (tester owns CDP).

## Verification

2026-11-06, tauri. /credentials renders header, stats (2 credentials), Add Credential and filter; no console errors; QA-043.mjs passes. After: shots/QA-043-after-credentials-header-only.png (cropped, no cards). Result: the 500 is gone. Add/edit/delete could not be verified: create fails, filed as QA-060; unlabeled icon buttons filed as QA-061.
