---
id: QA-160
title: "Vault unlock dialog appears on app load over unrelated pages and never says why"
status: verified
severity: minor
kind: ux
area: vault/VaultGate (all routes)
mode: tauri
found: 2026-11-22
---

## Use case

Open Portal and start working (projects, tasks, devices, settings).

## Steps

1. Cold-load any route (/projects, /tasks, /settings, /devices, /github) with the vault locked.
2. Dismiss with Not now, then navigate to GitHub.

## Expected

If the app must ask for the master password, the dialog says what needs it. Pages that do not need secrets do not prompt.

## Actual

A modal "Vault locked / Enter your master password" covers Projects, Tasks, Settings and Devices with no stated reason, so it reads like a random lock screen. After Not now it returns on /github. On /credentials the same state also shows inline ("Vault is locked - Unlock") so that page asks twice. The body copy says you will be asked again, but not by what. Lens: apple-design / make-interfaces-feel-better (explain why a permission is requested, at the moment of need).

## Evidence

`.claude/qa/shots/QA-160-before-vault-prompt-no-reason.png`

## Repro

manual - steps above

## Fix

Root cause: dialog had a generic body with no reason. Now every prompt carries a reason (startup: 'Unlock to use your saved credentials.'; Rust-triggered: 'An action needs your saved credentials.'; credentials banner/migration: specific). Re-prompt on /github is need-based, see QA-190. Github/ProjectEnv callers (other coders' files) left on the default reason. Files: src/lib/domains/credentials/state/vault.svelte.ts (ensureUnlocked(reason), reason cleared on settle), components/VaultUnlockDialog.svelte (shows the reason + 'skip, you'll be asked only when something needs it'), VaultGate.svelte and CredentialVault.svelte (pass reasons). Checks: pnpm check 0 errors.

## Verification

<!-- qa-tester -->
