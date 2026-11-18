---
id: QA-135
title: "AI > Code: composer silently fans out into parallel agents when the prompt has 2+ lines — a normal multi-line paste/prompt triggers multi-agent spawn with no confirmation"
status: fixed
severity: minor
kind: enhancement
area: ai/coder
mode: tauri
found: 2026-11-11
---

## Use case

A developer writes or pastes a multi-line prompt (a task with a couple of sentences on
separate lines, a pasted checklist) and expects one agent to work the task.

## Steps

1. `/ai/coder`, New session. Note the composer placeholder: "Ask the coder… (2+ lines
   auto-spawns parallel agents)".
2. Enter a prompt with 2+ lines and send.

## Expected

A multi-line prompt runs as one task; multi-agent fan-out is an explicit choice.

## Actual

- Any prompt with 2+ lines **auto-spawns one parallel agent per line** (multitask), with no
  confirmation. A normal multi-line paste or a task written across lines silently becomes N
  parallel agents. (In this trial I had to deliberately collapse the whole build request into
  a single line to avoid it.)

## Why it matters

Easy to trigger by accident; fanning out an unintended number of agents is costly and
confusing, and the trigger (a newline) is one every editor inserts naturally.

## Suggested fix

Make multi-agent fan-out explicit — a mode toggle or a "Run N tasks in parallel?" confirm
when 2+ lines are detected — rather than inferring it from the presence of a newline.

## Fix

Core footgun fixed by `8114329f`: `coderSession.svelte.ts` `looksLikeMultitaskInput` (`:1030-1039`)
now fans out on lines only when `agentMode === 'multitask'`, `multitaskMode` is set, or the thread
is a coordinator (default `agentMode` is `auto`, `:247`), and the empty-state placeholder
(`CoderPanel.svelte:734-738`) only advertises it in those modes. A plain multi-line prompt now
runs as one task.

Residual closed here: while a run was active, `CoderComposer.svelte` still showed "Queue
follow-up… (2+ lines spawns parallel agents)" in every mode. It now shows that hint only when
`agentMode === 'multitask'` (the condition the idle placeholder already uses) and a plain "Queue
follow-up…" otherwise.

Not built: the "Run N tasks in parallel?" confirm. Fan-out is now an explicit mode choice instead.
2+ GitHub issue URLs still fan out in any mode, by design (`:1031`). A coordinator thread in a
non-multitask agent mode still fans out on lines without the hint (the composer only receives
`agentMode`).

Checks: `pnpm vitest run src/lib/domains/coder` 14/14, `pnpm check` 0 errors, `pnpm lint` exit 0.
To verify (tauri): in Auto mode start a session and send a 2-line prompt: one agent runs, and
while it runs the composer placeholder is "Queue follow-up…" with no parallel-agents hint. Switch
the agent mode to Multitask: while running, the placeholder gains "(2+ lines spawns parallel
agents)".

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
