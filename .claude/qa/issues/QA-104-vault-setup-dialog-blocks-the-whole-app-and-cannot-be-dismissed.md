---
id: QA-104
title: "Set up your vault" dialog renders over every route, blocks all pointer interaction, and has no way to close it
status: verified
severity: blocker
kind: bug
area: credentials / vault
mode: tauri
found: 2026-11-09
---

## Use case

Do anything in the app while the vault has not been set up yet.

## Steps

1. Open the real app (vault not yet configured — "credentials already stored here" per the
   dialog copy, consistent with QA-049).
2. Navigate to any route: `/projects`, `/projects/audit`, `/pipeline-runs`, `/ai/coder`, `/tasks`.

## Actual

A "Set up your vault" `Dialog` (master password / confirm password / **Set up vault** button)
is rendered on top of the page, full-screen overlay (`fixed inset-0 z-50`). It:

- appears on every route (confirmed on `/projects`, `/projects/audit`, `/pipeline-runs`,
  `/ai/coder`, `/tasks` in the same session) and survives full `page.reload()`,
- has no close / X / "skip for now" control in the dialog markup,
- does not close on `Escape`,
- does not close on a click outside the dialog (the overlay intercepts the click instead of
  dismissing — confirmed via Playwright: a click on the backdrop left the dialog open and the
  next real click on page content timed out because `[data-dialog-overlay]` "intercepts pointer
  events").

With the dialog open, no button, link, row, tab or filter on the underlying page can be clicked
— the whole app is unusable except for typing into the two password fields. The only way found
to proceed was to actually fill in and submit the form, which this session declined to do since
it would set a real master password on the owner's real vault.

## Expected

A setup prompt for an optional feature (vault / credentials) should not block navigation and
interaction with the rest of the app. At minimum a "Not now" / close control, or confine the
prompt to the Credentials screen instead of mounting it globally.

## Why blocker

It cost this verification session the ability to drive the running app via Playwright/CDP for
a large part of a QA sweep — every route was unusable the moment the dialog had mounted. For an
ordinary user who is not ready to set a master password yet (or is unsure what it does), it is a
hard stop: the app cannot be used at all until they either set a password or restart it (and it
reappeared after a full reload, so even that may not clear it).

## Evidence

`.claude/qa/shots/QA-104-before-vault-dialog-blocks-app.png` (shown over `/projects`); the same
dialog is also visible in `.claude/qa/shots/QA-UX009-after-1280x800.png`,
`.claude/qa/shots/QA-UX009-after-1024x600.png` and `.claude/qa/shots/QA-UX010-after-click-run.png`,
taken over `/projects/audit` and `/pipeline-runs` in the same session.

## Repro

Manual — steps above. `document.querySelector('[data-dialog-overlay]')` is non-null and
`getByText('Set up your vault')` is visible on any route while the vault is unset.

## Fix

Root cause: `VaultGate.svelte` rendered `VaultSetupDialog` whenever `status?.state ===
'needs_setup'` — unconditionally, on every route, with no relation to whether anything had
actually asked to use a credential. The sibling `locked` branch already correctly gated on
`vaultGate.open` (only set when something calls `ensureUnlocked()` — the central
`invokeClient` retry-after-unlock path, the Credentials screen, or linking a project env var to
a credential); the `needs_setup` branch never had the same guard.

- `VaultGate.svelte`: condition is now `(status?.state === 'needs_setup' && vaultGate.open) ||
  vaultGate.setupStep !== null` — confines the setup prompt to when it was actually triggered,
  while still surviving the recovery-key reveal (unchanged `setupStep` logic).
- `VaultSetupDialog.svelte`: added a "Not now" button, enabled `showCloseButton` / Escape /
  outside-click-to-close, all gated on `step === 'password'` only — the one-time recovery-key
  reveal step stays forced (losing that key is unrecoverable), matching the existing
  `VaultUnlockDialog` dismiss pattern (`vaultGate.settle(false)`).

Files: `src/lib/domains/credentials/components/VaultGate.svelte`,
`src/lib/domains/credentials/components/VaultSetupDialog.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors). Also covers QA-049's area (credentials/vault) but
is a different file, as flagged.

## Verification

2026-11-09, mode: tauri. The owner's real vault is already set up (state=`locked`, not
`needs_setup` — confirmed on `/credentials`, 2 real GitHub credential entries present), so the
original full-block repro (dialog appearing unprompted on every route) can no longer occur in
this environment for the reason the fix intends: nothing ever sets `vaultGate.open` on a vault
that's already configured.

- `node .claude/qa/scripts/QA-104-check.mjs`: hard-loaded `/`, `/projects`, `/projects/audit`,
  `/pipeline-runs`, `/ai/coder`, `/tasks` (the exact routes from the original repro) — 0 dialog
  overlays, 0 "Set up your vault" text on any of them.
- `node .claude/qa/scripts/QA-104-dismiss.mjs`: confirmed the app is fully usable end-to-end
  (navigated to `/credentials` and back to `/projects`, page rendered normally, no stray
  overlay).
- Code read of `VaultGate.svelte` confirms the exact condition described in `## Fix`:
  `(status?.state === 'needs_setup' && vaultGate.open) || vaultGate.setupStep !== null`.
- Code read of `VaultSetupDialog.svelte` confirms the dismiss wiring claimed: password step has
  `showCloseButton={true}`, `interactOutsideBehavior="close"`, `escapeKeydownBehavior="close"`,
  and a "Not now" button calling `vaultGate.settle(false)`; the recovery-key step has all three
  set the opposite way (`showCloseButton={false}`, `"ignore"`, `"ignore"`) with no dismiss
  button at all, matching the claim that only the recovery-key reveal stays forced.

Could not safely trigger the actual `needs_setup` dialog live (would require tearing down the
owner's real configured vault, which is out of scope — real data), so the "Not now" click path
itself is verified by code read rather than a live click, per the task brief's own guidance.
Verified.

