---
id: QA-191
title: Vault unlock dialog is not centred, it sits in a corner of the window
status: needs-decision
severity: minor
kind: bug
area: credentials/vault, layout
mode: tauri
found: 2026-11-22
---

## Use case

Vault prompt appears; it should be a centred modal.

## Steps

1. Trigger the unlock dialog (cold load with locked vault, or /credentials -> Unlock).
2. Observe position at 1280x800 and 1024x600.

## Expected

Centred (stock `Dialog.Content`: `fixed top-[50%] left-[50%] translate -50%`).

## Actual

Reported by the owner: shown in a corner. Not yet reproduced by an agent. Suspect an ancestor of
`<VaultGate />` (`app-shell.svelte:270`) with `transform`/`filter`/`contain`/`backdrop-filter`
re-anchoring `fixed`, or a layout wrapper. `VaultUnlockDialog.svelte` adds only `sm:max-w-md`.

## Evidence

Tester/coder: screenshot first (`QA-191-before-vault-dialog.png`) and inspect computed ancestors.

## Repro

manual — steps above

## Fix

Cannot reproduce: with the vault dialog open at 1280x800 and 1024x600 over CDP, dialog-content is exactly centred (416,227 448x346 / 288,127) and no ancestor has transform/filter/contain/backdrop-filter/will-change/container-type. No file changed. Possibly only at cold load (before first layout) or a different webview zoom; need the owner's screenshot or window size. Screenshot saved as QA-191-before-vault-dialog.

## Verification

<!-- qa-tester -->
