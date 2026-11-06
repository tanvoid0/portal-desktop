---
id: QA-018
title: Rejected agent-platform token shows "Ready" in Settings and "Agent Platform is unreachable" in chat
status: verified
severity: major
kind: bug
area: ai/chat, settings/ai, ai/coder
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens AI chat/coder expecting to talk to agent-platform; something is wrong with the token and they need to know what.

## Steps

1. Platform is up (`curl http://127.0.0.1:18410/health` -> 200; `/v1/catalog` without token -> 401).
2. Settings > AI > Providers: badge reads "Ready". Click "Test connection" and "Load catalog".
3. Open `/ai/chat`.

## Expected

The status reflects reality: token rejected (401) -> "Token rejected, update it in Settings > AI" with a link. Chat/coder say
authentication failed, not "unreachable".

## Actual

- Real, not a fluke: the platform returns 401 for the configured token (Test connection logs
  "Authentication error: Invalid agent-platform API token"; Load catalog: "Invalid or missing agent-platform API token").
- Settings badge still says "Ready" (only checks a token is present).
- `/ai/chat` banner: "Agent Platform is unreachable / Failed to load catalog" with Retry, while the server is reachable.
  Retry cannot ever fix it. Footer: "Agent Platform is offline — retry to reconnect."
- `/ai/coder`: only a bare red "Catalog error" label with no explanation; provider list has just "Claude Code (CLI)".
- Console on every /ai page: "Failed to load live catalog: Authentication error: ..."
- No visible toast for the failed Test connection / Load catalog in the DOM (only logged by ToastStore); verify a toast actually shows.
- The token itself may simply be stale in the owner's config; the bug is the misleading status and copy.

## Evidence

`.claude/qa/shots/QA-018-before-chat-unreachable-2.png` and `sweep-ai-chat-initial.png` (banner), `sweep-ai-coder-initial.png` (Catalog error). Code: auth error mapped in
`agent_platform_provider.rs:111-115`; chat page treats any catalog failure as offline.

## Repro

`node .claude/qa/scripts/app.mjs ai/chat QA-018-after-chat` and read the banner.

## Fix

coder page part (done, status left): `CoderPanel.svelte` no longer shows bare "Catalog error"; it shows "Agent Platform token rejected. Settings > AI" (or "unreachable") with a link to /settings/ai, using ProviderModelSelector `onStatusChange` + `showInlineError={false}`. Not run in app. Settings status and chat still open.

Settings + chat part (done): root cause was the Settings badge only checking a token was present and chat treating every catalog failure as offline. Added isAuthError (ai/utils/catalog.ts); AIProviderSettings.svelte probes with testProvider on load and after Test/Load catalog, badge = Ready / Token rejected / Unreachable; routes/ai/chat/+page.svelte banner says token rejected with an Open Settings > AI button instead of Retry, and the composer hint explains why it is disabled. Toasts: ToastContainer is mounted in app-shell and store/Toast code looks sound; not verified live. Coder part review: uses `isAuthError`, `AI_PROVIDER_SETTINGS_PATH`, and the message is hidden when the CLI engine is selected.
Checks: pnpm check 0 errors, eslint/prettier clean on touched files.

Review follow-up: badge resets to unknown when api_key/base_url change; isAuthError reads .message from error objects; /settings/ai uses AI_PROVIDER_SETTINGS_PATH in chat page and ChatCatalogSelectors. pnpm check 0 errors.

## Verification

<!-- qa-tester -->

Also: with the catalog failing the chat composer textarea is `disabled` with no explanation beyond the footer, so chat send/stream could not be tested.
(`QA-018-before-chat-unreachable.png` is a blank frame captured during an HMR reload; use `-2`.)

2026-11-06, tauri. Coder header now says 'Agent Platform token rejected. Settings > AI' with link (shots/QA-018-after-coder-header.png). Chat banner: 'Agent Platform rejected the API token... Open Settings > AI', composer hint explains disabled state (shots/QA-018-after-chat-banner.png). NOT re-verified: Settings > AI badge and Test connection / Load catalog toasts; the auto-mode classifier blocked scripting that page (it shows the API token), so lead/owner should eyeball the badge (expect 'Token rejected') and whether a toast appears.

2026-11-06 (final round), tauri after rebuild. Settings > AI badge reads 'Token rejected' (token masked, never revealed, no shot of that page saved). Test connection shows a visible toast 'Connection test failed - Authentication error: Invalid agent-platform API token', so toasts do work. Chat banner 'rejected the API token' + Open Settings (shots/QA-018-after-final-chat-banner.png). Coder header verified earlier.
