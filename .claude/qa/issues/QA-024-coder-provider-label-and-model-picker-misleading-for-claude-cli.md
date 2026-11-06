---
id: QA-024
title: With "Claude Code (CLI)" selected the header still says "Agent Platform" and offers a model picker that is ignored
status: verified
severity: minor
kind: ux
area: ai/coder
mode: tauri
found: 2026-11-06
---

## Use case

Run the coder through the local Claude Code CLI engine.

## Steps

1. `/ai/coder`, Provider -> "Claude Code (CLI)" (the only option while the catalog fails, see QA-018).

## Expected

Label reflects the engine (no "Agent Platform"); no model/agent-mode controls that do nothing, or a note saying they are ignored.

## Actual

"Agent Platform" label stays next to the provider; "Select model" stays; mode chips "Auto" / "Auto" stay. Backend
(`coder/service.rs` `advance_claude_cli`) documents that model and Agent mode are ignored for this engine, and that
file changes / approvals are not populated, yet nothing in the UI says so.
Also: the chosen provider is not remembered; after a reload it is back to "Provider" (empty), and Ctrl+Enter then silently
does nothing useful until a provider is picked. Plain Enter inserts a newline and the only hint is the send button tooltip.

## Evidence

`.claude/qa/shots/sweep-ai-coder-claude-cli-fixture-selected.png`.

## Repro

manual — steps above.

## Fix

Root cause: coder UI did not model the Claude CLI engine, and provider choice lived only in memory.
Fix: when `claude-cli` is selected `CoderPanel.svelte` hides the "Agent Platform" label and model picker (new `showModel` prop on `ProviderModelSelector.svelte` / `ChatCatalogSelectors.svelte`, default true so chat is unchanged) and `CoderComposer.svelte` hides the permission/agent-mode chips (`hideModes`). Provider is persisted in localStorage `portal.coder.backendProvider` (`coderSession.svelte.ts`). Composer footer now always shows "Ctrl+Enter · send" (plus Shift+Tab hint when modes shown).
Review: Shift+Tab no longer cycles the hidden mode when the CLI engine is selected.
Checks: pnpm check 0 errors; eslint 0 errors, prettier clean on touched files. Not run in app (tester verifies).

## Verification

<!-- qa-tester -->

2026-11-06, tauri. With Claude Code (CLI) selected the 'Agent Platform' label, model picker and mode chips are hidden, footer shows 'Ctrl+Enter · send'; provider persists after reload (localStorage portal.coder.backendProvider). Reset to the original empty 'Provider' state afterwards. Shot shots/QA-024-after-claude-cli-selected.png.
