---
id: QA-020
title: /ai/logs is a placeholder: "This page was reset after a file encoding issue"
status: verified
severity: major
kind: bug
area: ai/logs
mode: tauri
found: 2026-11-06
---

## Use case

Owner wants to see AI request/response logs.

## Steps

1. Open `/ai/logs`.

## Expected

A working log view, or no route/nav entry if the feature is gone.

## Actual

Card text: "This page was reset after a file encoding issue. Rebuild log UI blocks here if needed." and
"Log features are temporarily unavailable in this view, but the route now builds correctly." Developer notes shown to the user.
`src/routes/ai/logs/+page.svelte` also starts with a BOM (U+FEFF) and uses double quotes / 2-space style that lint (prettier) may flag.
The page is rendered outside the AI tab shell sizing (title uses `PageHeader` inside bare `space-y-6`).

## Evidence

`.claude/qa/shots/sweep-ai-logs-loaded.png`.

## Repro

`node .claude/qa/scripts/app.mjs ai/logs QA-020-after-ai-logs`

## Fix

Restored the pre-reset page (last real impl in cff7d43: AILogsViewer + aiLogService, Rust commands ai_get_logs/ai_search_logs/ai_export_logs still registered) rewritten onto PageHeader, toast, no BOM, padded; breadcrumb now 'AI Logs' via `getAiTabBreadcrumb` in `src/lib/config/ai-tabs.ts`. Files: `src/routes/ai/logs/+page.svelte`, `ai-tabs.ts`. Not verified live. Checks: pnpm check 0 errors, eslint/prettier clean.

Review follow-up: Export now saves the returned JSON via Blob download and toasts 'Exported N logs'; logs load through a TanStack query keyed on filters (queryKeys.ai.logs) with PageLoading/PageError, so no race and no 'No logs found' while loading. Files: routes/ai/logs/+page.svelte, shared/query/keys.ts. pnpm check 0 errors.

## Verification

<!-- qa-tester -->

Also visible in the shot: no page padding (title and card touch the pane edges, card runs off the right edge), and the breadcrumb ends in "Home" instead of "AI Logs".

2026-11-06, tauri. /ai/logs renders under the AI shell with padding, breadcrumb 'AI Logs', search + provider (All, Ollama, Gemini) + type filters, empty state 'No logs found'; search with no match keeps empty state and shows Clear. Export not triggered. Shot shots/QA-020-after-ai-logs.png.

2026-11-06 (final round), tauri after rebuild. /ai/logs renders filters + 'No logs found' empty state (shots/QA-020-after-final-ai-logs.png). Clicked Export with zero logs: nothing happened, no toast, no file written (nit: no feedback; not filed).
