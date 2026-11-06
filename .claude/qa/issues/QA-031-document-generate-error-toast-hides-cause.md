---
id: QA-031
title: Document generation failure toast says "unexpected error" and the model picker only says "Catalog error"
status: verified
severity: minor
kind: copy
area: documents/generate, tasks/generate
mode: tauri
found: 2026-11-06
---

## Use case

AI generation fails (here: the known agent-platform token rejection, QA-018; this issue is only about how the failure is reported).

## Steps

1. Open `/documents/generate`, type a prompt, press "Generate Document".
2. Compare with `/tasks/generate` and "Generate Tasks".

## Expected

Both flows say what is wrong and where to fix it.

## Actual

- Tasks: toast "Failed to generate tasks: AI provider is not configured. Please set up Agent Platform in AI -> Providers." (good).
- Documents: toast "Failed to generate document: An unexpected error occurred during document generation". The real error (`Authentication error: Invalid or missing agent-platform API token`) is only in the console.
- Both pages: the model picker shows a bare red "Catalog error" with no reason or retry; Generate stays enabled although the provider cannot work.

The failure is visible (not a silent spinner), so this is copy/feedback only.

## Evidence

`.claude/qa/shots/sweep-documents-generate-attempt.png`, `.claude/qa/shots/sweep-tasks-generate-attempt.png`.

## Repro

Manual, steps above (no script: depends on the token being rejected).

## Fix

Root cause / change: Tauri rejects with strings; `parseError` only kept `Error` instances so the toast fell to the generic text. It now keeps string errors (toast shows the real "Authentication error: ..."). `ChatCatalogSelectors` "Catalog error" now uses `isAuthError` and the coder copy ("Agent Platform token rejected." / "unreachable." + Settings > AI link); also stringifies non-Error load errors. Not done: Generate stays enabled when the catalog fails (design call).
Files: src/lib/domains/documents/services/aiDocumentService.ts, src/lib/domains/ai/components/chat/ChatCatalogSelectors.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.

## Verification

2026-11-06, tauri mode (:9222). Generate Document now toasts the real 'Authentication error: Invalid or missing agent-platform API token' text; model picker shows 'Agent Platform token rejected. Settings > AI' with a link. Generate stays enabled (declared a design call). Shot: shots/QA-031-after-doc-generate-error.png. Verified.
