---
id: QA-016
title: Settings > AI shows the agent-platform API token in plain text
status: verified
severity: major
kind: ux
area: settings/ai
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens Settings > AI while screen-sharing, recording, or while QA takes screenshots.

## Steps

1. Open `/settings/ai` with a token configured.

## Expected

Token masked by default, with an eye button to reveal it (and hide again). Pasting or editing still works.

## Actual

The full `agp_live_…` token is shown in a plain text field ("Shown in plain text for
verification"). It leaked into two QA screenshots, which had to be deleted.

Owner decision 2026-11-06: mask by default and add a reveal toggle. Update the help text to
match.

## Evidence

Screenshots deleted for that reason. Field: `src/lib/domains/settings/` AI settings
component, "API token" input.

## Repro

manual: steps above

## Fix

Root cause: token Input was type=text. Now type=password with an eye/eye-slash icon Button (aria-label Show/Hide API token); help text updated. Used ~icons/fa6-solid icons because @lucide/svelte is not installed. File: src/lib/domains/ai/components/providers/AIProviderSettings.svelte. Checks: pnpm check 0 errors, eslint 0 errors, prettier clean on touched files.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
