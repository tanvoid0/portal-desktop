---
id: QA-045
title: GitHub shows "Connected" while every call fails with a raw decrypt error; only Retry offered
status: verified
severity: major
kind: bug
area: github
mode: tauri
found: 2026-11-06
---

## Use case

Browse repositories, issues and workflow runs of the connected GitHub account.

## Steps

1. Open `/github`: Account card shows `tanvoid0`, "Connected", scopes `read:user`, `repo`.
2. Open `/github/repos`, `/github/issues`, `/github/repos/<owner>/<repo>`, `/github/repos/<owner>/<repo>/actions/<id>`.

## Expected

If the stored token cannot be decrypted, the status should say so (needs reconnect) and the error state should offer "Reconnect GitHub" in plain words. Disabled create actions while unusable.

## Actual

All four pages show "Failed to decrypt GitHub credential: Decryption failed: ring::error::Unspecified" (raw crypto error leaked) with a Retry button that can never succeed. The home page still claims Connected. The repo detail page also renders "Link Existing Repo" / "Clone as Project" next to "Failed to load repository", and Issues keeps "New Issue" enabled. Possibly the same stored-key mismatch as credentials (see QA-043) - needs the coder to check. Not clicked: Disconnect/reconnect (out of bounds for QA).

## Evidence

`.claude/qa/shots/QA-045-before-github-decrypt-error-issues.png`, `sweep-github-repos-loaded.png`, `sweep-github-repo-detail-error.png`.

## Repro

`node .claude/qa/scripts/QA-045.mjs`

## Fix

Frontend done, Rust still proposed (left at `fixing`).
Root cause (not verifiable without touching the key): the GitHub token is a normal row in `credentials`, decrypted by the same `CredentialService::decrypt_credential` -> `get_master_key()` (credential_service.rs ~L246-300), whose 32-byte master key lives in the OS keychain under service `com.tanvoid0.portal-desktop`, entry `credential-master-key-v1`. `ring::error::Unspecified` is an AES-GCM tag mismatch = the ciphertext was sealed under a different key than the keychain now returns. `get_master_key` mints a fresh random key on `NoEntry` (after only checking LEGACY_APP_IDENTIFIERS), so the keychain entry was missing/re-created (or a rename not in the legacy list) after the token was stored. The key is not derived from machine id, key file or the pairing secret. Credentials list never decrypts (only reveal/use does), so QA-043's list rendering proves nothing about key health: any other credential will fail the same way on decrypt. No data/key touched.
Frontend: `GitHubError.svelte` (drop-in for PageError, detects "Failed to decrypt GitHub credential") shows "Stored GitHub token can't be read" + "Reconnect GitHub" -> /settings/github, no Retry; used on /github, repos, issues, repo detail (incl. issues tabs), workflow runs panel/monitor. Home page runs a 1-item `createGitHubTokenCheckQuery` and shows a "Needs reconnect" badge + button instead of "Connected". Link Existing Repo / Clone as Project (repo detail) and New Issue (issues) hidden/disabled when the token is unreadable.
Files: github/components/GitHubError.svelte (new), github/utils/tokenError.ts (new), github/queries.ts, github/index.ts, github/components/GitHubWorkflowRun{Monitor,sPanel}.svelte, routes/github/{+page,issues/+page,repos/+page,repos/[owner]/[repo]/+page}.svelte.
Checks: pnpm check 0 errors; eslint 0 errors; prettier clean.
Proposed Rust (optional, makes the status honest without a GitHub round-trip): in portal-core/src/domains/github/service.rs `get_connection_status` (L81-96), after the connection record is found, compute `token_readable` via `CredentialService::new(self.db.get_connection_clone()).decrypt_credential(&connection.credential_id).await.is_ok()` (discard the value), add `token_readable: bool` to `GitHubConnectionStatus` (+ `pnpm gen:types`), and set `connected: token_readable`-style handling in the UI; then `createGitHubTokenCheckQuery` can be dropped. Note decrypt_credential writes last_used; prefer a decrypt-only helper. Owner action to recover: Disconnect/reconnect GitHub (re-stores the token under the current key).

Lead (2026-11-06): set to `fixed` for what this issue reports (raw crypto error leaked,
"Connected" while unusable, dead Retry, actions offered that can't work); the frontend handles
all of that now. The underlying key loss is QA-049 (needs-decision). The optional Rust
`token_readable` status field above is not applied; the frontend probe covers it.

## Verification

2026-11-06, tauri, read-only. /github shows 'Needs reconnect' badge, 'Stored GitHub token can't be read. Reconnect GitHub to continue.' and a Reconnect GitHub button (not clicked). /github/repos and /github/issues show the same, no Retry; New Issue disabled. Root cause is QA-049. After: shots/QA-045-after-github-home.png, QA-045-after-github-issues.png. Result: gone.
