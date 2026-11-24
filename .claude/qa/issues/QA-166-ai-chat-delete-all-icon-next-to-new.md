---
id: QA-166
title: "AI chat: \"Delete all conversations\" is a lone trash icon beside \"+\", and most chats are titled \"Hello\""
status: verified
severity: minor
kind: ux
area: ai
mode: tauri
found: 2026-11-22
---

## Steps

1. Open /ai (Home tab).

## Actual

The Chats header has three icon-only buttons: new folder, new conversation, trash. The trash's only label is a tooltip ("Delete all conversations"), where users expect delete-selected. I did not click it (real data), so whether it confirms is untested. Five of eight chats are titled "Hello", distinguished only by count or date. Agent Platform selects read "Loading..." at first paint. Lens: apple-design (destructive actions separated and labelled), emil-design-eng.

## Evidence

`.claude/qa/shots/QA-166-before-ai-chat-delete-all.png`

## Repro

manual - steps above

## Fix

The trash already confirms via confirmAction (aiChatPage.handleDeleteAllConversations). Separated it from '+' with a left margin, added aria-label and 'asks to confirm' in the title. File: ai/components/conversations/ConversationList.svelte. Not changed: 'Hello' titles (auto-title generation = product call), 'Loading...' selects. Checks: pnpm check (0 errors), eslint+prettier on touched files.

Second pass: 'Hello' titles need no change - the new-chat flow already titles from the first user message (fallbackTitleFromMessage, 48 chars, in aiChatPage.svelte.ts); those chats' first message literally was 'Hello'. Loading selects: CatalogModelSelect gained loading (aria-busy) and aria-label props; ChatCatalogSelectors/ChatModelSelector pass 'Provider/Model (loading)' labels and the spinner has role=status. Files: ai/components/chat/{CatalogModelSelect,ChatCatalogSelectors,ChatModelSelector}.svelte. Checks: pnpm check 0 errors, eslint+prettier.

## Verification

<!-- qa-tester -->

2026-11-22 (round 2), tauri. Round 2: AI selects expose aria-label 'Provider (loading)' / 'Model (loading)' and aria-busy while loading.
