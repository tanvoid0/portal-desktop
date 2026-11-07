---
id: UX-018
title: Six components are too big to maintain — split plan with prop boundaries
status: fixed
severity: minor
kind: ux
area: settings / sdk / scripts / tasks / automation
mode: code-read + tauri
found: 2026-11-06
---

## Scope

The eleven largest components were examined. Four of them are **fine despite the line
count** and should be left alone; one is dead (UX-017); six need splitting. Each split below
names the files and the props, so there is nothing left to decide.

Do these one at a time — each is `pnpm check && pnpm lint` on its own.

---

## Leave alone

| file | lines | why it is fine |
| --- | --- | --- |
| [ProjectAudit.svelte](../../../src/lib/domains/projects/components/ProjectAudit.svelte) | 960 | One coherent workflow with a keyboard map that spans the whole view. The comments explain real decisions (WCAG target size at `:463`, the aria-live choice at `:534`, the 6-across breakpoint at `:553`). Splitting it would scatter the `j/k/1-4/x/esc` handler. Fix its **layout** (UX-009), not its size. |
| [CoderPanel.svelte](../../../src/lib/domains/coder/components/CoderPanel.svelte) | 852 | Already composed: `ResponsivePanel side="left"` at `:593`, `side="right"` at `:829`, `ChangesPanel` (477 lines) extracted. It is a three-pane app in one file because it coordinates three panes. |
| [sdk/ai/ollama/+page.svelte](../../../src/routes/sdk/ai/ollama/+page.svelte) | 880 | A five-tab single-purpose console. Split only if a second provider page appears. |
| [cloud/workloads/pods/[pod]/+page.svelte](../../../src/routes/cloud/workloads/pods/[pod]/+page.svelte) | 910 | Genuinely has more to show than its siblings (logs, exec, containers, events). It is the one cloud detail page that earns its length — see UX-013, which deliberately leaves the detail pages alone. |

---

## 1. SDKSidebar.svelte — 954 lines, two copies of the same sidebar

[src/lib/domains/sdk/components/SDKSidebar.svelte](../../../src/lib/domains/sdk/components/SDKSidebar.svelte)

Lines **602-760** render the icon-only sidebar. Lines **766-954** render the expanded one.
Same sections (Header, Navigation, SDK list, No-data fallback, Footer), same data, written
twice — including two separate "Loading SDKs…" blocks (`:645` and `:926`) and two "No SDKs
Detected" blocks (`:650`, `:931`).

The main app sidebar solves this with one markup tree and
`group-data-[collapsible=icon]:hidden` / `:justify-center` classes
([app-shell.svelte:312-345](../../../src/lib/components/layout/app-shell.svelte#L312)).

**Split:**

| new file | props |
| --- | --- |
| `SDKSidebar.svelte` | `{ selectedSDK?: string; navigationItemIds: string[] }` — keeps the query, renders the three parts below. Target ~120 lines. |
| `SDKSidebarNav.svelte` | `{ items: NavItem[]; currentPath: string }` |
| `SDKSidebarList.svelte` | `{ sdks: SdkConfig[]; selected?: string; loading: boolean; onSelect: (id: string) => void }` — owns the loading and empty states, **once**. |
| `SDKSidebarFooter.svelte` | `{ status: string }` |

Each renders one tree and uses `group-data-[collapsible=icon]:*` for the rail form. The
duplicate half disappears — expect ~400 lines removed, not just moved.

---

## 2. IdeSettings.svelte — 1187 lines, two settings pages in one

[src/lib/domains/settings/components/IdeSettings.svelte](../../../src/lib/domains/settings/components/IdeSettings.svelte)
— 25 `$state`, 28 functions, 18 Dialog usages, and two `<h2>` sections that have nothing to
do with each other: **IDE Configuration** (`:519-664`) and **Framework IDE Mappings**
(`:665-962`), plus two modals (`:963-994` Add/Edit IDE, `:995-…` Add/Edit mapping).

**Split:**

| new file | props |
| --- | --- |
| `IdeList.svelte` | `{ ides: Ide[]; onEdit: (i: Ide) => void; onDelete: (id: string) => void; onAdd: () => void }` |
| `IdeEditDialog.svelte` | `{ open: boolean (bindable); ide: Ide \| null; onSave: (i: Ide) => void }` |
| `FrameworkIdeMappings.svelte` | `{ mappings: Mapping[]; frameworks: Framework[]; suggested: Framework[]; onEdit; onDelete; onAdd }` |
| `FrameworkIdeMappingDialog.svelte` | `{ open (bindable); mapping: Mapping \| null; ides: Ide[]; onSave }` |
| `IdeSettings.svelte` | data loading + the two sections. Target ~150 lines. |

This is also the moment to fix the duplicate `<h2>IDE Configuration</h2>` the audit found
rendering twice on `/settings/ides`.

---

## 3. scripts/[id]/+page.svelte — 1031 lines, an editor and a runner behind one boolean

[src/routes/scripts/[id]/+page.svelte](../../../src/routes/scripts/[id]/+page.svelte)

`:525-690` is **Edit Mode** (Basic Information, Command, Parameters — the same three cards as
`/scripts/new`). `:691-1031` is **View Mode** (Execution History, Run Script, working
directory, interactive toggle, parameters, interactive terminal, output). An `isEditing`
boolean switches between them.

**Split:**

| new file | props |
| --- | --- |
| `ScriptForm.svelte` | `{ script: Script \| null; onSave: (s: Script) => void; onCancel: () => void }` — **shared with `/scripts/new`**, which today duplicates all three cards (494 lines). |
| `ScriptRunPanel.svelte` | `{ script: Script; onRun: (args: RunArgs) => void; running: boolean }` |
| `ScriptExecutionHistory.svelte` | `{ scriptId: string; onSelect: (executionId: string) => void }` |
| `scripts/[id]/+page.svelte` | header + mode switch. Target ~120 lines. |
| `scripts/[id]/edit/+page.svelte` | *new route* — renders `ScriptForm`. Drops the `isEditing` state entirely and makes the edit form linkable. |

Extracting `ScriptForm` removes the `/scripts/new` duplication at the same time.

---

## 4. tasks/[id]/+page.svelte — 965 lines

[src/routes/tasks/[id]/+page.svelte](../../../src/routes/tasks/[id]/+page.svelte)

Structure is already good (two columns; sidebar at `:801` holds Task Information, Linked
Document, Quick Actions). It is one file because nothing was extracted.

**Split:**

| new file | props |
| --- | --- |
| `TaskDetail.svelte` | `{ taskId: string }` — the whole body. **UX-012 needs exactly this** to render a task in the board's right rail. |
| `SubtaskList.svelte` | `{ parentId: string; subtasks: Task[]; onAdd; onEdit; onToggle; onDelete }` — `:562-800`, which carries its own add form and per-row edit form. |
| `TaskInfoCard.svelte` | `{ task: Task }` — `:803-853` |
| `TaskLinkedDocument.svelte` | `{ taskId: string; document: Document \| null }` — `:854-917` |
| `TaskQuickActions.svelte` | `{ task: Task; onGenerate; onArchive; onDelete }` — `:918-…` |

`tasks/[id]/+page.svelte` becomes `<TaskDetail taskId={id} />`, ~20 lines.

---

## 5. BlocksPage.svelte — 1088 lines

[src/lib/domains/automation/components/BlocksPage.svelte](../../../src/lib/domains/automation/components/BlocksPage.svelte)

Two tabs that share nothing but a page frame: Blocks (`:452-…`) and Templates (`:422` +
its `TabsContent`).

**Split:**

| new file | props |
| --- | --- |
| `BlockList.svelte` | `{ blocks: Block[]; search: string; category: Category \| null; onEdit; onExport; onDelete }` |
| `BlockEditDialog.svelte` | `{ open (bindable); block: Block \| null; onSave }` |
| `BlockImportDialog.svelte` | `{ open (bindable); onImport: (json: string) => void }` |
| `TemplateList.svelte` | `{ templates: Template[]; onApply; onDelete }` |
| `BlocksPage.svelte` | the single merged control row from UX-007 + the two lists. Target ~150 lines. |

Do UX-007 (the five control rows) in the same pass — it edits the same region.

---

## 6. sdk/+page.svelte — 867 lines

Covered by **UX-014**: deleting the three duplicate `TabsContent` blocks removes ~500 lines
with no extraction needed. Do UX-014 first and re-measure before splitting anything else.

---

## Fix

<!-- coder -->

## Verification

2026-11-07 — tauri, dev instance 2 (vite :1421 / CDP :9223, own database).
**Partial — not closing.** Three of the six splits were exercised, one was only sampled, two were
not reached.

**1. SDKSidebar (959 → 535, one CSS-driven tree).** Collapse and expand both work from the
right-hand "Toggle Sidebar": expanded 256px, rail 48px with the back arrow, four nav icons and six
category icons, no clipped labels in the rail. The rail tooltip fires ("Back to Portal Desktop").
The four Navigation items highlight the right one on `/sdk`, `/sdk/managers`, `/sdk/managers/nvm`,
`/sdk/services`, `/sdk/versions` and `/sdk/python`. Footer reads "11 of 16 SDKs", matching the
tree. Expanding a category by hand works and lists its SDKs. The deliberate changes read fine:
the header is a real back-to-Portal button and `Software Installer` / `Installations` are gone
from the nav, with the overview's Software Installer button reaching `/sdk/software-installer`.
Three defects filed: **QA-098** (the tree opens fully collapsed and the selected SDK's category is
never expanded — the headline behaviour of this component), **QA-099** (header subtitle clipped on
every `/sdk` page), **QA-097** (`/sdk` itself never finishes loading on a cold load). The mobile
sheet below 768px and the loading/empty states were not reached.

**2. IdeSettings (1187 → 47).** Full CRUD sweep — see the five issues filed from it: QA-094,
QA-095, QA-096, QA-102, QA-103. Add, edit, set-default and delete an IDE all work and persist;
Detect IDEs lists detected executables and a chip opens a prefilled Add dialog; mapping quick-add,
edit and delete work; deleting an IDE that had a mapping refreshed the mappings list to its empty
state; custom framework create (with DevIcon search), edit and delete work; the File icon branch
renders its picker button (I did not open the native dialog). Both empty states — "No IDEs
configured" with Detect/Add Manually, and "No framework IDE mappings configured" with the
quick-add chips — render correctly.

**3. scripts/[id] split, ScriptForm sharing, the new /edit route — NOT TESTED.** Ran out of
session budget before reaching it. Everything in that section is unverified.

**5. BlocksPage (1088 → 319).** Verified under UX-007 and QA-088; QA-100 and QA-101 filed.

Sections 4 and 6 were not in this sweep's scope and were not looked at.

