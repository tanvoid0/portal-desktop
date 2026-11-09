---
id: QA-086
title: Four more role="button" wrappers nest real buttons and swallow their labels
status: verified
severity: polish
kind: a11y
area: ai / terminal / tasks
mode: code-read
found: 2026-11-06
---

## Summary

`QA-083` was the coder changes panel. The same pattern — a `div role="button"` that
**contains** real `<Button>` elements and carries no `aria-label` — exists in four more
places. ARIA does not allow a nested `button` role, and the outer role's accessible name is
computed from its descendants, so each of these announces its children's button labels as
part of its own name and makes `getByRole('button', { name: ... })` ambiguous.

Found by grepping `role="button"` across `src/` after fixing QA-083 (6 files total; the
coder one is fixed, `SDKSidebarItem.svelte:60` is **fine** — it nests no interactive child).

## Sites

| File | Line | Nests | Risk |
| ---- | ---- | ----- | ---- |
| [TabBar.svelte](../../../src/lib/domains/terminal/components/TabBar.svelte#L186) | 186 | `<Button aria-label="Close tab">` | Worst of the four — the tab's name absorbs "Close tab", so a role query for the close button is ambiguous with the tab itself. |
| [AISessionCard.svelte](../../../src/lib/domains/ai/components/shared/AISessionCard.svelte#L76) | 76 | hover action button(s) | Card name absorbs the hover action's label. |
| [KanbanBoard.svelte](../../../src/lib/domains/tasks/components/KanbanBoard.svelte#L148) | 148, 293 | card action buttons | Card name absorbs status-toggle / add-subtask labels. |
| [TaskList.svelte](../../../src/lib/domains/tasks/components/TaskList.svelte#L61) | 61, 181 | row action buttons | Same for list rows. |

## Expected

Per site, the same fix applied in QA-083: drop `role="button"` from the wrapper, keep
`tabindex="0"`, give it `role="group"` with an explicit `aria-label`, so exactly one
accessible control exists per real action and the wrapper's name does not absorb its
children.

## Why it is worth doing

Beyond the a11y correctness, this is the failure mode that cost a QA run a false finding:
a role-based click resolves to the wrapper, lands on its focus/select handler, and
**silently does nothing** — no error, no console message. Any future driver script or
component test hits it. See `QA-083` for that write-up.

## Note on ownership

`KanbanBoard.svelte` and `TaskList.svelte` were rewritten and committed on 2026-11-06 by a
concurrent session ("Tasks UI revamp and organization") in this same checkout. They were
deliberately **not** edited here to avoid a concurrent write; that session has been told.
`TabBar.svelte` and `AISessionCard.svelte` are uncontested and are the cheapest two to do
first.

## Fix

Root cause: `div role="button"` nesting real interactive children — ARIA has no nested-button
role, and the wrapper's accessible name absorbs its children's labels. Applied the QA-083
pattern (drop `role="button"`, keep `tabindex="0"`/`onkeydown`, add `role="group"` + an
explicit `aria-label`):
- `src/lib/domains/terminal/components/TabBar.svelte` (tab row → `role="group"` + `aria-label="Tab: {tab.title}"`)
- `src/lib/domains/ai/components/shared/AISessionCard.svelte` (card → `role="group"` + `aria-label={title}`)

`KanbanBoard.svelte`/`TaskList.svelte` were already fixed by the concurrent "Tasks UI revamp"
session (confirmed via grep — both now use `role="group"`), so not touched here.

Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: code-read (matches how this was filed). Confirmed via grep: `TabBar.svelte`
line ~186 and `AISessionCard.svelte` line ~76 both now use `role="group"` with the exact
`aria-label` described. `KanbanBoard.svelte` (lines 148, 294) and `TaskList.svelte` (lines 61,
182) also both use `role="group"`, confirming the concurrent session's fix is still in place.
A full `role="button"` grep across `src/` turns up only `ChangesPanel.svelte` (QA-083, already
fixed) and `SDKSidebarItem.svelte` (named fine in this issue, nests no interactive child).
Verified.
