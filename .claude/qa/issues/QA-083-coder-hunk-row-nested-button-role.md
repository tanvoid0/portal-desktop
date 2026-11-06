---
id: QA-083
title: Coder changes panel wraps the per-hunk Accept/Reject buttons in a role="button" div
status: open
severity: polish
kind: a11y
area: coder / ChangesPanel
mode: tauri
found: 2026-11-06
---

## Steps

1. AI -> Code, engine **Claude Code (CLI)**, workspace `todo-rn`.
2. Start a session and ask for a one-line edit, e.g. _"In App.tsx, change the heading text from Todos to My Todos. Nothing else."_
3. When the turn finishes, open the Workspace rail -> **Agent changes**.
4. Inspect the hunk row (`hunk 1 · line 63`) with the accessibility tree, or query it:
   `document.querySelectorAll('[role="button"] button').length`

## Actual

The hunk container at [ChangesPanel.svelte:380](../../../src/lib/domains/coder/components/ChangesPanel.svelte#L380)
is a `<div role="button" tabindex="0">` with its own `onclick` / `onkeydown` (they set
hunk focus), and the real **Accept hunk** / **Reject hunk** `<button>` elements are nested
**inside** it.

So one hunk exposes three overlapping controls to assistive tech, and the outer one's
accessible name is computed from all of its descendant text:

```
role=button  name="hunk 1 · line 63 Accept hunk Reject hunk - <Text style={styles.title}>Todos</Text> + ..."
  role=button  name="Accept hunk"
  role=button  name="Reject hunk"
```

Consequences:

- ARIA does not allow a nested `button` role inside a `button` role; the inner buttons
  are in the outer button's forbidden content model.
- A screen reader announces the whole diff — both button labels included — as the name of
  the focusable row, before the user ever reaches the real buttons.
- `getByRole('button', { name: 'Reject hunk' })` resolves to the **container**, not the
  button, so the click lands on the focus handler and silently does nothing. This cost a
  QA run a false "per-hunk reject is broken" finding before the element was inspected
  (`tagName` came back `DIV`). Any role-based automation or test hits the same trap.

## Expected

The hunk row is focusable without claiming to be a button:

- drop `role="button"` from the container and keep `tabindex="0"` (a `role="group"` with
  an `aria-label` of just `hunk 1 · line 63` would also do), or
- move the Accept/Reject buttons out of the focusable container.

Either way, exactly one accessible control per actual action, and the row's accessible
name should not swallow the diff text.

## Not a bug

Per-hunk accept/reject itself **works**. Clicking the real `<button>` reverted
`App.tsx` on disk and moved the row to `status=rejected` with `hunks[0].accepted=0`.
Only the element layering is wrong.

## Evidence

- `.claude/qa/shots/coder-A-agent-changes-diff.png`
- `.claude/qa/shots/coder-A-after-real-reject-hunk.png`
