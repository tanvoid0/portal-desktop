# Regression checklist: /tasks board (pre-rewrite behaviour)

Baseline = git HEAD (`TaskManager`, `KanbanBoard`, `TaskList`, `TaskFilterModal`, `taskUi.svelte.ts`, `dragDrop.ts`, `taskStore.ts`).
Written 2026-11-06, mode: tauri (real backend, CDP).

**Evidence grading.** The parallel rewrite hot-reloaded into the live app mid-run, so the old board was only
screenshotted once (`shots/sweep-tasks-board-baseline.png`). Tags:

- `(seen)` observed live in the app (detail/create/edit/generate pages, backend behaviour, the old board before it was swapped)
- `(code)` derived by reading HEAD; NOT observed running. Re-run in the new UI and trust the new UI.
- `[BROKEN NOW]` already defective in the old board; the rewrite may change it.

Fixtures: create `QA Alpha` (High, Bug, description contains `needle-alpha`), `QA Beta` (Feature), `QA Gamma` (Low, no type).
The board holds the owner's real tasks: never touch a task not named `QA ...`. Delete fixtures at the end.
Subtask = `/tasks/create?parentId=<id>`.

## Board (kanban)

- [ ] Columns left to right: To Do, In Progress, Completed, Cancelled; each header shows a count of its cards. `(code; baseline shot shows the first three + Cancelled 0)`
- [ ] Only parent tasks are cards. A subtask is never its own card; it renders nested/indented under its parent, in the PARENT's column whatever the subtask's own status. `(code; seen in rewrite)`
- [ ] Card shows: title, priority flag + PRIORITY word (HIGH/MEDIUM/LOW), description (2-line clamp), `#tags`, type badge, subtask count (arrow icon + n) when it has subtasks. `(seen)`
- [ ] Drag a card onto another column: status becomes that column's status, persists across reload, other cards untouched. `(code; seen working in rewrite)`
- [ ] Drag a card onto its own column: no-op, no request. `(code)`
- [ ] Dragged card is dimmed while dragging; hovered column gets a ring highlight and loses it on leave. `(code)`
- [ ] Dropping on Cancelled sets status `cancelled`. `(code)`
- [ ] Drag a parent that has subtasks: only the parent's status changes; subtasks keep their own. `(code)`
- [ ] A failed move gives NO user feedback. `[BROKEN NOW]` `handleTaskMove` only `console.error`s; no toast. `(code)`
- [ ] Every status change / drag flips the global `$isLoading`, so the whole board is replaced by the "Loading tasks..." spinner for a beat (flash; scroll and focus lost). `[BROKEN NOW]` `taskStore.updateTask` sets `isLoading`, `TaskManager` gates the board on it. `(code)`
- [ ] Status-toggle button on a card (circle / progress-clock / check / cancel icon, coloured) cycles `pending -> in-progress -> completed -> cancelled -> pending` (4 steps, wraps). Same button on each nested subtask. `(seen on the detail page "Toggle Status": To Do > In Progress > Completed > Cancelled > To Do)`
- [ ] Status-toggle click does NOT open the detail page (stopPropagation). `(code)`
- [ ] "+" on a parent card = create subtask, goes to `/tasks/create?parentId=<id>`. Subtasks have no "+". `(code)`
- [ ] The create-subtask form shows NO hint that it is a subtask (header "Create New Task", nothing about the parent). `[BROKEN NOW, minor]` `(seen)`
- [ ] After saving the subtask: lands on the SUBTASK detail page, which shows `Parent Task: View Parent Task`; the parent card on the board now shows count 1 and the nested row. `(seen)`
- [ ] Click a card -> `/tasks/<id>`. Click a nested subtask -> `/tasks/<subtaskId>` and does not also open the parent. `(code)`
- [ ] Enter / Space on a focused card (tabindex 0, role=button, aria-label "Select task: <title>") = click. `(code)`
- [ ] Empty column shows just header + 0 count, no placeholder. `(seen: Cancelled 0)`

## List

- [ ] Kanban/List toggle in the toolbar; default Kanban; choice is NOT persisted across reload or navigation. `(code)`
- [ ] One row per parent task, subtasks as indented rows below. Row: status toggle, "+" (parents only), title, subtask count, description, `#tags`, type badge, PRIORITY badge, status badge (To Do / In Progress / Completed / Cancelled). `(code)`
- [ ] Row click -> detail; subtask row click -> subtask detail; Enter/Space works. `(code)`
- [ ] Status toggle and "+" behave exactly as on the kanban card. `(code)`
- [ ] Search/filters affect the list the same as kanban (same filtered set). `(code)`
- [ ] Switching view keeps search, filters and selection. `(code)`

## Selection (multi-select)

- [ ] "Multi-Select" button enters the mode and is replaced by an amber bar: `n selected` badge, Select All, Clear, Delete (n), Cancel. `(code; button seen in baseline shot)`
- [ ] In the mode every card/row shows a checkbox. Clicking the card body toggles selection instead of navigating; clicking the checkbox toggles only (no double toggle, no navigation). Selected card gets a warning ring/tint. `(code)`
- [ ] Entering OR leaving the mode clears the selection. `(code)`
- [ ] Select All selects every currently VISIBLE (filtered) parent task: not hidden ones, not subtasks. `(code)`
- [ ] Clear empties the selection and stays in the mode; Cancel leaves the mode. `(code)`
- [ ] Delete (n) is disabled at 0 selected. `(code)`
- [ ] Bulk delete asks first: "Are you sure you want to delete N selected task(s)? Also deletes M subtask(s)." (M part only when >0; destructive "Delete" button). Cancel keeps everything. `(code)`
- [ ] Confirming deletes the tasks AND their subtasks (backend cascade, no orphans): afterwards `/tasks/<subtaskId>` shows "Task not found". `(seen via single delete)`
- [ ] After bulk delete the success toast reports the right count. `[BROKEN NOW]` The toast reads `selectedTaskIds.size` after the store already cleared the selection, so it says "Successfully deleted 0 tasks". `(code)`
- [ ] After bulk delete the selection is empty. The old board stays in multi-select mode afterwards. `(code)`
- [ ] A failed bulk delete shows an error toast "Failed to delete tasks". Deletes run in parallel (`Promise.all`), so a partial failure leaves some deleted and the toast does not say which. `[BROKEN NOW, minor]` `(code)`
- [ ] Single-task delete lives on the detail page: Delete -> confirm "Are you sure you want to delete "<title>"? Also deletes N subtask(s). This action cannot be undone." -> Esc cancels (stays on the page) -> Delete returns to `/tasks`. `(seen)`

## Filters

- [ ] Search box (placeholder `Search tasks...`) filters live as you type, case-insensitive, on title OR description. Does NOT match tags, type, or subtask text. `(seen: `needle-alpha` found the card by description; `qa beta` matched case-insensitively; `sub 1` found nothing although the subtask is visible nested under its parent)` Subtask-text miss: `[BROKEN NOW, product call]`.
- [ ] Filters button opens an "Advanced Filters" dialog: Search, Status (To Do, In Progress, Completed, Cancelled), Priority (Low, Medium, High), Types (Story, Bug, Feature, Note, Task, Epic). Buttons: Clear All (red), Cancel, Apply Filters. `(code)`
- [ ] Dialog edits are staged: the board changes only on Apply. Cancel/close discards. Clear All clears everything and closes. `(code)`
- [ ] Reopening the dialog shows the currently applied filters pre-ticked. `(code)`
- [ ] Within a group OR (status in [a,b]); across groups AND (status AND priority AND type AND search). `(code)`
- [ ] Type filter only offers the six hard-coded types; tasks with another or empty type cannot be targeted and drop out as soon as any type is ticked. `(code)`
- [ ] Filters button shows a count badge = number of active GROUPS (status, priority, type, search each count 1), not number of values. `(code)`
- [ ] Active-filter chips row under the search card when anything is set: `Active filters:` + chip per search text, per status, per priority (coloured flag), per type; each chip's x removes just that value; `Clear All` removes everything including the search box text. `(code)`
- [ ] Stats tiles (Total / To Do / In Progress / Done) and the Progress bar count ALL parent tasks and ignore filters. `(seen: baseline 15 / 5 / 3 / 7, 47%)`
- [ ] No tasks at all: "No tasks yet" + "Create Task" action. Filters match nothing: "No matching tasks" + "Clear filters" action, which clears search and chips and restores every card. `(seen)`
- [ ] Load error: "Failed to load tasks" with Retry. `(code)`
- [ ] Filters are page-local: leaving /tasks clears them; not persisted. `(code)`
- [ ] Esc inside the search box does NOT clear it. `(seen in rewrite; no handler at HEAD)`

## Navigation

- [ ] "New Task" -> `/tasks/create`; "Generate Tasks with AI" -> `/tasks/generate` (renders Story Text, Custom Instructions, Generate Tasks, Refine Tasks, chat panel). `(seen)`
- [ ] Create a task (title alone is enough; footer hint: Ctrl+Enter saves, Ctrl+Escape cancels): lands on the NEW task's detail page `/tasks/<id>`; the task is on the board afterwards. `(seen)`
- [ ] Create form fields: Title*, Description, Status, Priority, Type, Due Date, Estimated Time, Actual Time, Tags, Assignee, Linked Document, Advanced Settings (recurring, dependencies). `(seen)`
- [ ] Detail `/tasks/<id>`: Back to Tasks, status badge, Edit, Delete, details card, Parent Task link for subtasks, Quick Actions (Toggle Status, Edit Task), hint "Ctrl+E to edit, Ctrl+S to save, Esc to cancel". `(seen)`
- [ ] Edit `/tasks/<id>/edit`: loads current values; Ctrl+Enter saves and returns to the detail page; the change shows there. `(seen)`
- [ ] Back from detail returns to `/tasks`; view/filters are NOT restored. `(code)`
- [ ] Missing task (`/tasks/<deleted id>`): "Failed to load task / Task not found / Retry". `(seen)`
- [ ] Opening a card also calls `navigator.vibrate(50)` where supported (no-op on desktop). `(code)`

## Keyboard (window-level `keydown`; they also fire while typing in an input)

- [ ] `Ctrl+N` -> `/tasks/create`. `(code)`
- [ ] `Ctrl+F` -> opens the filter dialog (overrides webview find). `(code)`
- [ ] `Ctrl+K` -> focuses the search box (selector `input[placeholder="Search tasks..."]`). `(code)`
- [ ] `Ctrl+M` -> toggles multi-select mode. `(code)`
- [ ] `Ctrl+A` -> select all visible tasks, ONLY in multi-select mode; it hijacks "select all text" in the search input. `(code)`
- [ ] `Ctrl+H` -> opens the "Keyboard Shortcuts" dialog (also the "Shortcuts" toolbar button). It lists Ctrl+N/F/K/M, Ctrl+A, Delete, Ctrl+H, Esc and has "Got it". `(code)`
- [ ] The shortcuts dialog documents plain `Delete` = "Delete selected", but the handler sits inside the Ctrl/Meta branch, so plain `Delete` does nothing and only `Ctrl+Delete` works. `[BROKEN NOW]` `(code)`
- [ ] `Escape` closes, in priority order: filter dialog, shortcuts dialog, then leaves multi-select mode. `(code)`
- [ ] Shortcut handlers are removed when navigating away from the board. `(code)`
- [ ] Dead code at HEAD, safe to drop: sidebar tabs (QuickActions, SavedViews, TimeTracker, TemplateManager), the `TaskCard` import in TaskManager, helpers duplicated per component. `(code)`

## Limits

- The old board was only seen once; every `(code)` line must be confirmed in the new UI, not assumed.
- Drag was exercised via Playwright `dragTo` (works against the rewrite), not a real mouse drag in WebView2.
- /tasks/generate: render only, no AI call made.
