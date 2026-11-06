---
id: QA-086
title: Four more role="button" wrappers nest real buttons and swallow their labels
status: open
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
