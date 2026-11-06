---
id: QA-065
title: Editing a credential saves, but the form logs "Cannot read properties of null (reading 'id')" and shows an error toast
status: verified
severity: minor
kind: bug
area: credentials
mode: tauri
found: 2026-11-06
---

## Use case

Rename/update a credential.

## Steps

1. `/credentials`, create `QA credential` (fake), Edit it, change name, enter a new value (Value is required on edit), Update.

## Expected

Dialog closes, "Credential updated" toast, no errors.

## Actual

The change persists (visible after reload) and the dialog closes, but the console shows
`Failed to save credential Cannot read properties of null (reading 'id')` and `Toast Error: Cannot read properties of null (reading 'id')`.
Likely `update_credential` returns null (Option) and the code dereferences `.id`.
Also: Edit requires re-typing the Value ("Value is required") just to rename; confirm that is intended.

## Evidence

Console errors above, from the verify run of QA-060 (2026-11-06).

## Repro

manual — steps above (QA-created credential only).

## Fix

Root cause: `onSave` closes the form and nulls the `credential` prop, then the success logger read `credential.id` (TypeError -> error toast though saved). Id is now captured before the save. Value is optional on edit (empty sends undefined; backend `update_credential` only re-encrypts `if let Some(value)`, so no Rust change needed). Removed the form's duplicate success toast (vault already toasts).
File: credentials/components/CredentialForm.svelte. Checks: as QA-064. Not run live.

## Verification

<!-- qa-tester -->

2026-11-06 (last round), tauri. Renamed without entering a Value: dialog closed, rename persisted after reload, zero console errors.
