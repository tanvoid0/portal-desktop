---
id: QA-162
title: "Credentials: two identical \"GitHub (tanvoid0)\" cards, fake \"encr****\" value, card actions clip at the edge"
status: verified
severity: minor
kind: ux
area: credentials
mode: tauri
found: 2026-11-22
---

## Use case

Find which stored credential is which.

## Steps

1. Open /credentials.

## Expected

Entries are distinguishable; locked values show an obvious placeholder.

## Actual

Two cards share name, type and description; only "Last used" differs (on one only). The Value box reads "encr****", which looks like real content rather than a locked placeholder. The eye/pencil/trash icons sit against the card's right edge (trash touches it); delete is adjacent to edit with no labels, and the user cannot tell which card to delete. Lens: apple-design (clarity), make-interfaces-feel-better (alignment).

## Evidence

`.claude/qa/shots/QA-162-before-duplicate-credential-cards.png`

## Repro

manual - steps above

## Fix

Root cause: card printed maskValue('encrypted_value') = 'encr****' as a fake value; header actions could shrink and clip; no field to tell duplicates apart. Fix: italic 'Hidden — use the eye icon to reveal' placeholder, actions shrink-0 with title truncate, added an 'Added: <date>' line. File: src/lib/domains/credentials/components/CredentialCard.svelte. pnpm check 0 errors. Underlying duplicate GitHub rows (data) not touched.

## Verification

<!-- qa-tester -->
