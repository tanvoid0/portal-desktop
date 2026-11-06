---
id: QA-060
title: Add Credential always fails, with no message: create_credential called with snake_case args
status: verified
severity: blocker
kind: bug
area: credentials
mode: tauri
found: 2026-11-06
---

## Use case

Add a credential to the vault.

## Steps

1. Open `/credentials`, "Add Credential".
2. Name `QA credential`, Value `qa-fake-value` (fake), click Create.

## Expected

Credential saved, dialog closes, "Credential created" toast, card appears.

## Actual

Dialog stays open, no toast, nothing saved (total stays 2 after reload). Console: `invalid args credentialType for command create_credential: command create_credential missing required key credentialType`. `credentialService.createCredential` (`credentialService.ts` ~L36) sends `credential_type` and `expires_at`; Tauri expects camelCase (`credentialType`, `expiresAt`) unless the command sets rename_all. `updateCredential` sends `expires_at` too, so likely the same. `CredentialForm` only logs the error: no `toast.error`, so the user gets no feedback. Found while verifying QA-043; update and delete could not be tested because no fixture could be created (existing credentials not touched).

## Evidence

`.claude/qa/shots/QA-060-before-create-credential-fails.png` (fake value only), console lines above.

## Repro

`node .claude/qa/scripts/QA-060.mjs`

## Fix

Root cause: `credentialService` passed snake_case top-level invoke args (`credential_type`, `expires_at`); Tauri expects camelCase (`credentialType`, `expiresAt`). Fixed in create and update (the only snake keys). `CredentialForm` already had an inline error but showed a generic message for Tauri string errors; it now shows the real message, toasts it, and the dialog stays open with input intact.
Files: credentials/services/credentialService.ts, credentials/components/CredentialForm.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean. Not run live.

Same-class sweep: Fixed (top-level keys to camelCase): deploymentService.ts:611-613 `build_docker_image_command` (contextPath, imageName, dockerfilePath), deploymentService.ts:644 `get_process_status_command` (deploymentId), ideService.ts:200 `set_framework_ide_mapping` (ideId), tauriTaskService.ts:237 `get_subtasks` (parentId). Rust params were plain snake_case with no rename_all, so these calls failed with missing key. Confirmed fine: `download_and_install_version` (sdkService.ts:744, sdk/versions/+page.svelte:81): Rust takes only sdk_type and version; `use_manager` is an extra arg Tauri ignores, so camelCasing it is pointless (dead arg, left alone). Second grep of src/ for snake_case top-level invoke keys found nothing else outside device auth. Device auth (NOT changed, needs owner review): `generate_device_passcode` (deviceAuthService.ts:163) Rust takes a single `_request: GeneratePasscodeRequest` struct, so the frontend's flat device_id/device_name/device_info fails with missing key `request` today (nested struct fields are snake_case, so wrapping as `{ request: {...} }` would be the fix). `verify_device_passcode`, `get_device_status`, `verify_access_token` (deviceAuthService.ts:191/242, AuthScreen.svelte:95, DeviceAuthGuard.svelte:73) no longer exist as Tauri commands (moved to daemon /pair/* routes, see network/commands.rs header), so these invokes fail with unknown command regardless of arg casing; callers' effect depends on their catch handling, which was not changed.

## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. Created 'QA credential' via Add Credential: dialog closed, card appeared. Edit saved and persisted (but see QA-065: console null-id error and Value required on rename). Delete confirm dialog appears but delete does not reach the backend (QA-064). Fixture removed via invoke delete_credential; none left. shots/QA-060-after-create-credential.png.
