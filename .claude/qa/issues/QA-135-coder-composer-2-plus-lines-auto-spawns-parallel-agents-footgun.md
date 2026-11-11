---
id: QA-135
title: "AI > Code: composer silently fans out into parallel agents when the prompt has 2+ lines — a normal multi-line paste/prompt triggers multi-agent spawn with no confirmation"
status: open
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

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
