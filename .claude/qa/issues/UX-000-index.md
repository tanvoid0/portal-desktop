---
id: UX-000
title: UX sweep index — density, right sidebars, information architecture
status: open
severity: minor
kind: ux
area: all
mode: tauri (:9222, 1280x800 and 1024x600)
found: 2026-11-06
---

Eighteen findings from a page-by-page sweep of 93 routes. Does **not** repeat the five
density causes already assigned in parallel (global `h-14` header, `page-header.svelte`,
`page-stats.svelte`, nested `container mx-auto p-6`).

Not tested: `/cloud/*` beyond the connection guard (no cluster on this machine — UX-013 says
so explicitly) and `/github/*` list content (no repos synced; the shells render).

Two right-rail primitives already exist and are in use — `ResponsivePanel side="right"`
(fixed width, `Sheet` below `md`; `CoderPanel.svelte:829`, `ai/chat/+page.svelte:612`) and
`ResizablePaneGroup` (drag-resizable; `TerminalWorkspace.svelte:208`). **No new primitive is
needed and `shell-sidebar-layout.svelte` does not need a right variant.**

## Ranked by win ÷ effort

| # | id | win | effort | severity |
| --- | --- | --- | --- | --- |
| 1 | UX-001 dashboard greeting + 7 duplicate tiles | 381px, half the fold | delete 2 blocks | major |
| 2 | UX-017 FlyEnvStyleDashboard is dead code | −1049 lines | delete 2 lines | minor |
| 3 | UX-004 `TabsList w-full` on 13 pages | labels stop floating in 400px cells | delete a class ×13 | minor |
| 4 | UX-003 19 hand-rolled `text-3xl` headers | ~60px × 19 + 100px on /deployments | mechanical | major |
| 5 | UX-002 13 hand-rolled stat-card grids | ~100px × 13 | mechanical | major |
| 6 | UX-005 settings nav 993px + double heading | ~160px/page, nav stops scrolling | one component + 16 h2 deletions | major |
| 7 | UX-007 blocks: 5 control rows, 300px cards | 134px + 4× blocks per screen | one component | major |
| 8 | UX-009 audit scan form above the results | 0 → 7 triage rows at 1024x600 | right rail | major |
| 9 | UX-006 log viewer 9 lines where 25 fit | 9 → ~25 lines | one component | major |
| 10 | UX-016 form headers + section subtitles | ~160px/form, 5 forms consistent | mechanical | minor |
| 11 | UX-012 tasks duplicate counts + detail nav | 44px + board stops clipping | 1 + 3 (needs TaskDetail) | minor |
| 12 | UX-008 projects renders 184 rows / 11731px | 15 screens → ~2, 190 → ~10 tab stops | collapsible groups | major |
| 13 | UX-011 project detail 3 header rows | 192px → ~100px + facts stay visible | right rail | major |
| 14 | UX-014 /sdk tabs duplicate 3 routes | −500 lines, one place per thing | IA | major |
| 15 | UX-010 pipeline run detail leaves the list | removes the `?from=` hack | right rail + extract | major |
| 16 | UX-015 settings frameworks hub duplicates 3 routes | 16 → 12 routes | IA | minor |
| 17 | UX-013 10 cloud list routes are one file | 20 → 4 route files | IA, largest | major |
| 18 | UX-018 six oversized components | ~1000 lines removed, not moved | per-component | minor |

---

## A. Right-sidebar candidates

Strict list. A right rail earns its place only where it beats the current layout — three
pages do, one is a facts rail, and the rest do not.

**UX-009 · `/projects/audit` — the strongest case.** Triage is a repetitive keyboard loop
(`j/k · 1-4 · o · x · esc`) and at 1024x600 **no triage row is on screen at all**; at
1280x800 exactly one is. Rail (`w-80`): (1) the whole 250px "Folders to scan" form, (2) the
row under the `j/k` cursor — path, size, last commit, unpushed count, remote, duplicate
siblings, which today needs leaving the page, (3) the `x`-marked count and the four bulk
decision buttons. Main column goes from 1 row to 11 at 1280, 0 to 7 at 1024.

**UX-010 · `/pipeline-runs` — removes a hack, not just pixels.** Clicking a run does
`goto('/projects/<p>/pipelines/run/<e>?from=' + encodeURIComponent(list))`. The `?from=`
parameter exists only because the detail cannot sit next to the list (and is what QA-078's
breadcrumb bug surfaces). Rail (`w-[28rem]`): status, duration, step list, log output —
the existing route's body, extracted as `PipelineRunDetail.svelte { executionId }`. Mirror
the selection as `?run=<id>`; keep the route for deep links.

**UX-011 · `/projects/[id]` — a facts rail, not a list rail.** Rail (`w-80`):
`ProjectTechStack` and `ProjectGitCard`, both already prop-driven, so it is a move not a
rewrite. They stay visible while you work in the Actions and Terminal tabs instead of being
scrolled past. Main column keeps `ProjectCommands` and `ProjectModules` — where the
clicking happens.

**UX-012 · `/tasks` — conditional.** Worth it only together with narrowing the project rail
to `w-56`: 870px of content minus a 416px rail leaves 454px, under two 300px columns.
Default the rail closed, open on card click. Needs `TaskDetail.svelte { taskId }` from
UX-018 §4 first.

**Rejected:** `/logs` (needs density, not a rail — UX-006), `/automation/blocks` (needs a
table, not a rail — UX-007), `/credentials` and `/documents` (too few items to justify
splitting the width), `/settings` (already two-column; the problem is the left nav),
`/cloud/*` (judge after UX-013 collapses it). `/terminal`, `/ai/coder` and `/ai/chat`
already have one and are the reference implementations.

---

## B. Per-page compaction / reorder — ordered by win

1. **UX-001 `/`** — delete the `text-4xl` greeting (82px) and the Quick Actions card (299px,
   7 of 7 tiles duplicate the sidebar). Keep New Project / New Task as header buttons.
   Workspace summary moves from y=672 to ~y=190.
2. **UX-003** — 19 files hand-roll `<h1 class="text-3xl font-bold">` + description instead
   of `PageHeader` (12 SDK, 5 cloud, `DeploymentDashboard`, `FlyEnvStyleDashboard`), so the
   parallel `page-header.svelte` compaction never reaches them. Plus `/deployments` spends
   100px on a Card holding "Docker is ready. Container operations are available."
3. **UX-002** — 13 files hand-roll a `md:grid-cols-4` stat grid; `PageStats` has **one**
   caller. `/credentials` holds two credentials and spends 118px on "Total 2 / Active 2 /
   Expired 0 / Types 1". The six cloud workload pages have byte-identical copies.
4. **UX-005 `/settings`** — nav Card is 278×**993px** with 71px two-line items whose
   descriptions restate their labels ("Terminal / Terminal configuration"); it scrolls at
   1280x800. Plus a 44px reserved badge row showing a hardcoded `4/7`, plus every page
   stating its title twice (top bar "Settings" + body "General Settings"). One-line items →
   all 14 sections visible; ~160px back per page.
5. **UX-007 `/automation/blocks`** — 238px of chrome in five stacked rows, three of them
   near-empty; page named three times (breadcrumb, tab, h1). Cards are 300px tall for
   `Type: command` / `Version: 1.0.0` — constants on all 17 blocks. Merge to one control row,
   96px cards.
6. **UX-006 `/logs`** — 9 lines visible where ~25 fit: 44px rows, three nested frames
   (page h1 → Card "Unified Logs" → an inner bordered box), a hardcoded
   `h-[calc(100vh-12rem)]`, a footer bar. Make it a fill-viewport monospace stream.
7. **UX-016 forms** — `/tasks/create` and `/documents/create` have **no `<h1>` at all**; the
   other three hand-roll one three different ways. Four `CardDescription`s on `/scripts/new`
   restate their own titles (160px), and a collapsed `Import from JSON` accordion owns the
   top 120px.
8. **UX-008 `/projects`** — 184 rows, 33 groups (11 of them single-project), **11 731px** of
   scroll, 190 buttons, no collapse, no virtualisation. Collapsible groups + hover-only row
   menu.
9. **UX-012 `/tasks`** — the three count badges are repeated verbatim by the column headers
   44px below; `min-w-[300px]` columns clip the third at 1280 and push 8 controls past the
   right edge at 1024.
10. **UX-004** — `TabsList class="w-full"` / `grid-cols-N` on 13 pages defeats the vendored
    `w-fit` default, so three labels float across 1172px.

---

## C. Information architecture

### UX-013 · Cloud: 20 route files → 4

`diff cloud/workloads/daemonsets/+page.svelte cloud/workloads/statefulsets/+page.svelte` =
**87 changed lines of ~200**, and all 87 are the resource's name, its `ResourceType` and its
column array. Ten list pages, one file's worth of logic — each carrying its own copy of
UX-002 and UX-003.

```
before                                           after
/cloud/workloads                                 /cloud/workloads          (overview, unchanged)
/cloud/workloads/{pods,services,deployments,     /cloud/[resource]
  jobs,cronjobs,daemonsets,statefulsets}         /cloud/[resource]/[name]
/cloud/workloads/{...}/[name]                    /cloud/[resource]/new
/cloud/{configmaps,secrets,ingress}
/cloud/{configmaps,secrets,ingress}/[name]       + redirects from every old path
/cloud/{configmaps,secrets}/new
```

`ResourceType → { label, columns, canCreate }` lives in
`src/lib/domains/cloud/core/resourceDescriptors.ts`; the command palette
(`cloud/+layout.svelte:57-90`, which hardcodes 4 of the 10) generates from it. Detail pages
are a **separate, later** pass — `pods/[pod]` genuinely earns its 910 lines.

### UX-014 · SDK: `/sdk` tabs duplicate three routes

`TabsContent value="managers" | "services" | "versions"` render lists inline **and** the
same lists exist as `/sdk/manager` (318 lines), `/sdk/services`, `/sdk/versions` (249) with
different markup off the same queries. Plus a third route to the same places via the 954-line
`SDKSidebar`.

```
before: /sdk (4 inline tabs) + /sdk/manager + /sdk/services + /sdk/versions
after:  /sdk (overview only) + /sdk/managers + /sdk/services + /sdk/versions
        /sdk/manager → redirect to /sdk/managers
```

Deletes ~500 of `/sdk/+page.svelte`'s 867 lines. Sidebar becomes the only navigation.

### UX-015 · Settings: 16 routes → 12

`/settings/frameworks-languages?tab=languages` and `/settings/languages` render the same
component, and the nav offers both (parent → hub, children → standalone routes). Same shape
for `/settings/ai` (13 lines) over `/settings/ai/training`.

```
delete /settings/frameworks-languages  → redirect, ?tab= maps to the matching route
delete /settings/ai/+page.svelte       → redirect to /settings/ai/providers
nav: "Frameworks & Languages" and "AI" become group labels, not clickable sections
```

### UX-017 · Delete `FlyEnvStyleDashboard.svelte`

1049 lines; the only reference in `src/` is its own barrel export. Nothing renders it. It
carries a copy of UX-002, UX-003 and UX-004 that every future codemod pays for.

### Considered and rejected

- Merging `/logs` into `/terminal` — different jobs; `/logs` is the app's own log, not a shell.
- Merging `/utilities/disk` + `/utilities/environment` into one page — they already share a
  layout and a tab bar; two routes is correct.
- Merging `/deployments` into `/cloud` — local Docker and a k8s cluster are different things
  and the owner's Docker is live while the cluster is not.
- A new `/dashboard` route — UX-001 makes `/` worth keeping.

---

## D. Components too big to maintain (UX-018)

**Already fine despite the line count — do not split:**

| file | lines | why |
| --- | --- | --- |
| `ProjectAudit.svelte` | 960 | one keyboard workflow spanning the whole view; its comments document real decisions. Fix its **layout** (UX-009), not its size |
| `CoderPanel.svelte` | 852 | already composed — two `ResponsivePanel`s and `ChangesPanel` extracted; it coordinates three panes |
| `sdk/ai/ollama/+page.svelte` | 880 | one five-tab console; split when a second provider appears |
| `cloud/workloads/pods/[pod]/+page.svelte` | 910 | the one cloud detail page with genuinely more to show (logs, exec, containers) |

**Split, with the prop boundary (full detail in UX-018):**

| file | lines | split into | boundary |
| --- | --- | --- | --- |
| `SDKSidebar.svelte` | 954 | `SDKSidebarNav` / `SDKSidebarList` / `SDKSidebarFooter` | lines 602-760 and 766-954 are **two copies** of the same sidebar (icon-only and expanded), incl. two "Loading SDKs…" and two "No SDKs Detected". One tree + `group-data-[collapsible=icon]:*`, as `app-shell.svelte:312` already does → ~400 lines **removed** |
| `IdeSettings.svelte` | 1187 | `IdeList` / `IdeEditDialog` / `FrameworkIdeMappings` / `FrameworkIdeMappingDialog` | two unrelated `<h2>` sections (`:519` IDE config, `:665` framework mappings) + 2 modals; 25 `$state`, 18 Dialogs |
| `scripts/[id]/+page.svelte` | 1031 | `ScriptForm` / `ScriptRunPanel` / `ScriptExecutionHistory` + a new `scripts/[id]/edit` route | an editor (`:525-690`) and a runner (`:691-1031`) behind one `isEditing` boolean; `ScriptForm` is **shared with `/scripts/new`**, which duplicates all three cards today |
| `tasks/[id]/+page.svelte` | 965 | `TaskDetail {taskId}` / `SubtaskList` / `TaskInfoCard` / `TaskLinkedDocument` / `TaskQuickActions` | layout is already right; nothing was extracted. `TaskDetail` is the prerequisite for UX-012's right rail |
| `BlocksPage.svelte` | 1088 | `BlockList` / `BlockEditDialog` / `BlockImportDialog` / `TemplateList` | two tabs sharing only a page frame; do it in the same pass as UX-007 |
| `sdk/+page.svelte` | 867 | — | **no extraction**: UX-014 deletes the three duplicate `TabsContent` blocks, ~500 lines. Re-measure after |
| `FlyEnvStyleDashboard.svelte` | 1049 | — | delete (UX-017) |
