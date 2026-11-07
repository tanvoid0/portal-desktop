# Handoff — Portal Desktop tasks board revamp

Written 2026-11-06. For someone with no memory of the session that produced it.

Note: `.claude/handoff.md` is a _different_ session's handoff, covering the coder engine work.
This file is separate on purpose — do not merge them.

## Goal

The owner's complaint about `/tasks`: visual bugs, too cramped, impractical, completed tasks
pile up forever. They wanted scrumban-style progressive disclosure, grouping by project, and
projects in a sidebar.

## Status: the revamp is DONE, committed and verified

Committed on branch `feat/coder-cli-permission-mode` (nothing pushed):

- `8530cbb` — feat(tasks): revamp the tasks board, list and archive (16 files)
- `e524574` — chore: ignore the generated `.impeccable` hook cache in prettier
- `f53cd46` — docs(qa): the QA sweep record

Gates on the committed tree, verified by reading real exit codes: `pnpm check` exit 0,
`0 ERRORS`; `pnpm lint` exit 0; 216 portal-core tests pass.

### What shipped

- Columns are **Backlog / In Progress / Done**. Cancelled is a filter, not a column.
- **Archive**: completed >7 days old plus all cancelled. Grouped by completion month,
  searchable, Restore button per row. **Nothing is ever deleted** — the owner was explicit
  about this. The archive rule is derived from `completedAt`, so there is no `archived`
  column and no migration.
- **Project sidebar** via `ShellSidebarLayout`. Tasks link to projects through
  `resourceType === 'project'` + `resourceId === String(project.id)` (written by the portal
  MCP). Only projects that actually have tasks are listed — there are 180+ registered.
  Selection lives in the URL (`?project=<id|all|none>&view=board|list|archive`).
- **Progressive disclosure** on cards: title and priority flag always; type, due date,
  blocked, subtask count, assignee, estimate and project label only when set.

## Outstanding work

### 1. Board #89 / QA-086 — card ARIA (the live decision)

The owner was asked "fix now or leave it" and the session ended before they answered.

`KanbanBoard.svelte:148` and `:293`, `TaskList.svelte:61` and `:181` each use
`<div role="button">` as a wrapper that **contains** real `<Button>` children (status toggle,
add-subtask). Two faults:

- ARIA forbids nested button roles.
- The wrapper computes its accessible name from descendants, so a screen reader announces the
  card text plus every nested button label, and `getByRole('button', { name })` is ambiguous.

This has teeth beyond a11y: a role-based click resolves to the wrapper, hits its click handler
and **silently does nothing**. It produced a false "per-hunk reject is broken" finding in
another session before they checked `tagName` and got `DIV`.

Proven fix, already green in `ChangesPanel.svelte` (commit `e2bb5a8`), ~2 lines per site:
drop `role="button"`, keep `tabindex="0"`, add `role="group"` + an explicit `aria-label`,
expose selection via `aria-current` rather than only a ring colour.

It predates the revamp (the old wrapper divs had it) but was carried onto the new Cards.
Needs a11y judgement, not a blind attribute removal — the div keeps a click handler and must
stay keyboard-operable.

### 2. Untested, not broken

- **Archive month grouping across >1 month has never been exercised.** The backend stamps
  `completed_at` from the real clock; Playwright's `page.clock` only moves frontend time;
  cancelled tasks have a null `completed_at` and group by `updatedAt` — also November.
  Producing a second month needs a seeded DB row or a test hook. I deliberately refused to
  write fake rows into the owner's real Portal database. The grouping code is type-checked
  but unproven.
- **`/github/issues` is untested.** It shares `KanbanBoard`/`TaskList`, so the refactor
  touched it, but its stored GitHub token cannot be read and it renders no board. Needs the
  owner to reconnect GitHub before anyone can verify closed/open styling there.

### 3. Other board items filed, not mine to close

- **#78** — `cargo clippy` fails on a pre-existing logic bug: `if !in_link_text || true` at
  `src-tauri/portal-core/src/domains/projects/discovery/scan.rs:619`. The `|| true` makes the
  guard dead. Not a safe blind delete — it changes what the link scan skips. Blocks clippy as
  a gate.
- **#73** — two unconfirmed UI observations from another session (a click near the AI tab
  strip navigating to Tasks; the collapsed icon rail scrolling its bottom icons out of view).
  Ruled out as regressions from this work: `isShellSidebarRoute` has one caller and the new
  entry only fires on `pathname === '/tasks'`.

## Files that matter

| Path                                                              | Why                                                                      |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `src/lib/config/layout-breakpoints.ts`                            | `SHELL_SIDEBAR_EXACT_ROUTES = ['/tasks']`. **This is the cropping fix.** |
| `src/lib/domains/tasks/components/TaskManager.svelte`             | Sidebar, toolbar, URL state, keyboard handler                            |
| `src/lib/domains/tasks/components/KanbanBoard.svelte`             | Height-bound board, per-column scroll                                    |
| `src/lib/domains/tasks/components/ArchiveView.svelte`             | Month grouping + Restore                                                 |
| `src/lib/domains/tasks/state/taskUi.svelte.ts`                    | `boardTasks`, `archivedTasks`, `projectCounts`, archive rule             |
| `src/lib/domains/tasks/utils/taskDisplay.ts`                      | Shared status/priority/icon helpers                                      |
| `src-tauri/portal-core/.../tasks/repositories/task_repository.rs` | `completed_at` transition logic + its test                               |
| `src/routes/github/issues/+page.svelte`                           | Second consumer of the board components                                  |
| `.claude/qa/REGRESSION-tasks-board.md`                            | Behaviour checklist                                                      |
| `.claude/qa/issues/`                                              | QA-076…086                                                               |

## Things that bit us — read these before repeating them

1. **The cropping bug was height, not width.** The board already had `overflow-x-auto`.
   Nothing bounded the page, so it grew to ~3700px and the horizontal scrollbar sat below the
   fold. Three plausible CSS hypotheses were all wrong; only measuring `scrollWidth` vs
   `clientWidth` up the ancestor chain found it. **Measure before patching layout.**

2. **`min-w-0` must go the whole way down a flex chain.** A first fix added it to the title
   and its immediate parent; one wrapper above still grew to the text width, so `truncate`
   did nothing. QA caught it.

3. **Don't run a QA sweep while anything edits the tree or rebuilds the app.** It happened
   twice. HMR swapped the UI mid-walk, and later a Rust rebuild relaunched the app under a
   run — its results straddled a restart and had to be thrown away. **Record the
   `portal_desktop.exe` pid at start, mid-run and end and abort if it changes.**

4. **`pnpm check | tail -3` reports `tail`'s exit code, not the gate's.** Another session
   shipped 4 svelte-check errors while claiming green. Redirect to a file and read `$?`, or
   read svelte-check's own `COMPLETED … 0 ERRORS` line.

5. **Running `cargo test` regenerates 90 ts-rs binding files** under `src/lib/domains/*/generated/`
   with double quotes, undoing prettier. Revert them (`git checkout --`) before committing.

6. **`getByRole('button', { name })` is unsafe in this app** — see #89 above. Prefer
   `locator('button').filter({ hasText })`.

7. **Portal MCP `update` notes are capped at ~200 chars** and fail with a confusing
   "Task title must be less than 200 characters" error.

8. **Verify a peer session's claims against the repo.** Their reports were accurate but a
   file list sent from this session omitted two new untracked files and would have broken the
   build had they not caught it. List untracked files too, not just tracked changes.

## Update — #89 partially fixed (2026-11-06, later)

All four sites changed from `role="button"` to `role="group"` + `aria-label` + `aria-current`
(`KanbanBoard.svelte`, `TaskList.svelte`). Gates verified by exit code: `pnpm check` exit 0,
`0 ERRORS`; `pnpm lint` exit 0. **Uncommitted.**

What this fixed:

- The invalid nested-button-role violation is gone.
- `getByRole('button', { name })` no longer resolves to the wrapper, so role-based clicks in
  QA scripts stop silently no-opping. This was the part with teeth.

**What it did NOT fix — read before closing #89.** The change trades a hard ARIA violation
for four new svelte-check warnings (146 → 150), two per file:
`a11y_no_noninteractive_tabindex` and `a11y_no_noninteractive_element_interactions`. A
`role="group"` div carrying `tabindex=0`, `onclick` and `onkeydown` is still not a correct
interactive control — the element announces as a group but behaves as a button.

The properly correct fix is a card redesign, not an attribute swap: make the card body
non-interactive and put the primary action on a real `<button>` wrapping the card title, with
the existing action buttons as siblings rather than descendants. That removes the warnings and
the violation together. It was not done here because it changes card layout and markup in
both components, which deserves its own change and its own QA pass.

A gotcha that cost a cycle: an HTML comment placed between attributes inside a tag is parsed
as an attribute, producing `attribute_duplicate` / `expected_token` errors and a cascading
"has no default export" in every importer. Put explanatory comments above the element, never
inside the attribute list.
