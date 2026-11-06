---
id: QA-030
title: AI generate pages: "Refine" chat header overflows the right edge, horizontal scrollbar
status: verified
severity: minor
kind: ux
area: tasks/generate, documents/generate
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens "Generate Tasks with AI" or "Generate with AI" and wants to pick a provider and model for the refine chat.

## Steps

1. Open `/tasks/generate` or `/documents/generate` at the default 1280x800 window.

## Expected

The chat panel header (title, Provider select, model select) fits its column; no horizontal scroll.

## Actual

The "Refine Tasks" / "Refine Document" title is squeezed into a narrow wrapped column while the Provider select is pushed past the window edge (right edge at 1273 px on tasks, 1299 px on documents in a 1280 px window; cut off on documents). The content scroller scrolls horizontally (1048 > 1014 and 1075 > 1014) and a horizontal scrollbar shows at the bottom.

## Evidence

`.claude/qa/shots/QA-030-before-tasks-generate-chat-header.png`, `.claude/qa/shots/QA-030-before-documents-generate-chat-header.png`, `.claude/qa/shots/sweep-documents-generate.png`.

## Repro

`node .claude/qa/scripts/QA-030.mjs`

## Fix

Root cause / change: AIChatPanel header: title + two fixed-width selects (140+280px) plus "Agent Platform" label did not fit a 1/3 column. Header now wraps, title shrink-0, platform label hidden, selects 120/170px. Shared by tasks and documents generate (ai domain, not tasks).
Files: src/lib/domains/ai/components/chat/AIChatPanel.svelte
Gates: pnpm check 0 errors; eslint 0 errors and prettier clean on touched files. Not rerun against the app (told not to touch :9222); needs tester verification.

## Verification

2026-11-06, tauri mode (:9222). Both /tasks/generate and /documents/generate: no horizontal scroller, Provider select right edge 1068 px in 1280 px window, title on one line. After shots: shots/QA-030-after-documents-generate.png, shots/QA-030-after-tasks-generate.png. Verified. (Not rechecked at 1024px.)
