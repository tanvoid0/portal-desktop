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

Second pass (live dev app, CDP :9222, event `vault://unlock-requested` emitted to open the gate): still cannot reproduce.
Tried: rAF sampling of the dialog rect every frame for 120 frames from open at 1280x800, 1024x600, 800x500, 2560x1080 (0 off-centre samples, centred even mid zoom-in); cold reload with an init script sampling from document start through 4s with the gate opened as soon as the shell mounted (206 samples, 0 off-centre); html zoom 0.5/1.25/1.5/2 with deviceScaleFactor to match (centred each time). Portal is Svelte-portalled to body by bits-ui, no re-parenting seen. Only oddity: at zoom 2 the dialog (736px) is taller than a 700px viewport and its top clips at -18px (stock Dialog.Content has no scroll/max-height); not a corner. No code changed, no defensive fix (no evidence). Repro: `node .claude/qa/scripts/QA-191.mjs` prints ok. Needs the owner's screenshot/window size to go further.

## Verification

<!-- qa-tester -->

2026-11-22 defensive change check, tauri (status unchanged): unlock dialog centred at 1280x800 (cy 419 of 839 high) and 800x500; at an equivalent 200% zoom (viewport 640x250) it gets overflow-y auto, scrolls (scrollHeight 366 > 216) and Not now is reachable. Shots QA-191-after-vault-dialog-*.png. CSS html zoom:2 is not a valid emulation (viewport units do not shrink).
