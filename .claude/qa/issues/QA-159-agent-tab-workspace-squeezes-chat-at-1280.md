---
id: QA-159
title: Agent tab with Files/Git panel open squeezes the chat column to ~220px at 1280x800
status: open
severity: minor
kind: ux
area: ai/coder/claude-code (AgentsWorkspace)
mode: tauri
found: 2026-11-17
---

## Use case

Read the agent transcript and reply while browsing the session's files or git changes.

## Steps

1. Open `/ai/coder/claude-code?session=<id>` at 1280x800 (session list 320px + app nav visible).
2. Workspace sidebar is open by default; click Files (or Git changes).

## Expected

Chat stays usable next to the panel (or the panel/sidebar yields space).

## Actual

Three fixed columns (session list w-80, workspace panel, sidebar w-56) leave the chat ~220px wide: the Ended/Transcript/Timeline tabs are clipped ("nv" fragment of a tab cut off) and the composer wraps "Send a message to resume..." over three lines.

## Evidence

`.claude/qa/shots/sweep-agent-tab-files.png`

## Repro

`node .claude/qa/scripts/QA-870-verify.mjs` then view the shot.

## Fix

## Verification
