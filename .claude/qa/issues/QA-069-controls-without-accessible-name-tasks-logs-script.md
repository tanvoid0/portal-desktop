---
id: QA-069
title: Icon-only buttons, switches and selects with no accessible name on tasks kanban, logs filters and script detail
status: verified
severity: minor
kind: a11y
area: tasks, logs, automation/scripts
mode: tauri
found: 2026-11-06
---

## Use case

Operate the page with a keyboard / screen reader (and hover tooltips for sighted users).

## Steps

1. `node .claude/qa/scripts/QA-069.mjs`.

## Expected

Every icon-only control has `aria-label` (and ideally a tooltip).

## Actual

- `/tasks` (kanban): the status-toggle circle on every task card is a nameless button (35 of them with 11 tasks; the svg is `aria-hidden`).
- `/logs`: three filter comboboxes all read just "all" - no label says what each filters (level / source / context).
- `/scripts/<id>` (script detail): the back arrow button, the "Interactive Mode" switch and a checkbox have no name. Interactive Mode is also ON by default, so a plain `echo hello` opens an "Interactive Terminal" rather than just running (see description "Enable for scripts that require user input").

## Evidence

`.claude/qa/shots/QA-069-before-unnamed-controls-logs.png`, `QA-069-before-unnamed-controls-script-detail.png`, `sweep-tasks-default.png`, `QA-069-before-unlabeled-task-buttons.png`.

## Repro

`node .claude/qa/scripts/QA-069.mjs`

## Fix

Names added: `TaskCard.svelte` + `KanbanBoard.svelte` status-toggle/create-subtask buttons and select checkbox; `UnifiedLogViewer.svelte` three filter selects via a new `aria-label` prop on `ui/select.svelte`; script detail back button, Interactive Mode switch (id + Label for) and boolean-parameter checkbox. Verified by accessible name: /tasks 0 unnamed, logs selects read Filter by level/source/context, switch labelled Interactive Mode. (QA-069.mjs's heuristic ignores `<label for>` so still lists the logs auto-scroll checkbox and the switch; both are labelled.) Interactive Mode default (`interactiveMode = $state(true)` in `scripts/[id]/+page.svelte`) left ON - looks wrong against its own help text, needs an owner call.

## Verification

<!-- qa-tester -->
