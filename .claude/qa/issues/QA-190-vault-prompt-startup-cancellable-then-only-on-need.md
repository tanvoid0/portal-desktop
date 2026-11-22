---
id: QA-190
title: Vault prompt: keep the cancellable startup ask, re-prompt only when a screen needs the vault
status: verified
severity: minor
kind: ux
area: credentials/vault
mode: tauri
found: 2026-11-22
---

## Decision (owner, 2026-11-22)

KEEP the startup prompt (`app-shell.svelte:111` `vaultGate.promptIfLocked()`), and keep it
cancellable ("Not now"). After a cancel, the prompt must NOT reappear until the user is on a
screen/action that actually needs the vault unlocked. Do not remove the startup call.

## Use case

Cold load on any route with a locked vault: one prompt, dismissible. Then browse freely.
Opening something that needs the key (credential reveal/create/update, GitHub connect, project
env values, DB explorer credentials) prompts again, once.

## Expected

- Startup: single dialog, says why ("Unlock to use saved credentials. Skip, you'll be asked
  when something needs it"), cancel settles `false`.
- After cancel: no prompt on Projects, Tasks, Settings, Devices, etc.
- Needs-vault screens/actions call `ensureUnlocked()` / hit `require_key` and prompt then.

## Actual (from QA-160)

Prompt returns on /github after "Not now" (may be legitimate, check it is gated on a real vault
need, not on mount). /credentials asks twice: dialog plus inline banner. Dialog gives no reason.

## Steps

1. Lock vault, restart, land on /projects, click Not now. Browse 5 routes: no re-prompt.
2. Go to /credentials, /github, a project with env: each prompts at most once, only on need.

## Fix

Audit: startup `promptIfLocked` kept (cancellable, settles false). No screen prompts on mere mount; the only re-prompts are need-driven: Rust `require_key` (decrypt/create/update credential; /github decrypts the stored token to call GitHub, so its prompt after Not now is a real need, not mount), `ensureUnlocked` in GitHub connect/GitHubError/ProjectEnv/CredentialVault button. Status refetch does not reopen (open only set by ensureUnlocked). The /credentials inline banner is the need-based affordance, not a second dialog. Fix is the missing 'why' copy. Files: src/lib/domains/credentials/state/vault.svelte.ts (ensureUnlocked(reason), reason cleared on settle), components/VaultUnlockDialog.svelte (shows the reason + 'skip, you'll be asked only when something needs it'), VaultGate.svelte and CredentialVault.svelte (pass reasons). Checks: pnpm check 0 errors.

## Verification

<!-- qa-tester -->
