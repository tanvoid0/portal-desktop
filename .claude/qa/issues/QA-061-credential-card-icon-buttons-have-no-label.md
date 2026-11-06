---
id: QA-061
title: Credential cards: reveal, edit, delete and copy icon buttons have no accessible name
status: verified
severity: minor
kind: a11y
area: credentials
mode: tauri
found: 2026-11-06
---

## Steps

1. Open `/credentials`, inspect the icon buttons on a credential card (reveal value, edit, delete, copy).

## Expected

aria-label and tooltip on each ("Show value", "Edit credential", "Delete credential"), same as QA-046.

## Actual

`CredentialCard.svelte` ~L165-196: ghost icon Buttons with no aria-label or title. Delete is destructive and icon-only.

## Evidence

Script prints 6 unnamed buttons for 2 cards (no secret shown).

## Repro

`node .claude/qa/scripts/QA-061.mjs`

## Fix

Added aria-labels with the credential name to reveal/hide, edit, delete and copy buttons in credentials/components/CredentialCard.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean.

## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. Cards expose aria-labels: 'Reveal/Hide value of QA credential', 'Edit ...', 'Delete ...', 'Copy value of ...' (copy appears after reveal). QA-061.mjs: unnamed buttons 0.
