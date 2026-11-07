# Handoff — UX compaction: title bars, page density, information architecture

Written 2026-11-07. For someone with **no memory** of the session that produced it.

Note: `.claude/handoff.md` (coder engine) and `.claude/handoff-tasks-board.md` (tasks board
revamp) are **different sessions' handoffs**. Do not merge them into this one or this one
into them.

## Goal

The owner's words: pages are "too spacious for small screens, and cluttered", space is
wasted, "it includes the global title bar, and per app title bar", no page uses a right
sidebar where one would help. Review page by page, operate the pages rather than just
screenshot them, compact and reorder. **It is a developer-first app** — density is a
feature, not a regression.

Scope widened twice by the owner mid-session:

1. Split oversized components into smaller atomic ones, and run it as **two agents, one
   reviewing, one fixing**. Later refined to a strict **reviewer → fixer → tester**
   pipeline, each with only the context its own duty needs, because a fixer grading its own
   work is not a check. This is saved in memory as `agent-pipeline-review-fix-test`.
2. Freedom to create, reorganise, merge or remove pages — but the owner asked to **approve
   any page merge, move, delete or new route first**. They approved four (UX-013, UX-014,
   UX-015, UX-017); anything further needs a route map before/after and their yes.

## The measurement that started it

Real app over CDP at 1280x800, main content area 744px. Pixels burned before the first
content row: `/logs` 180, `/settings/general` 210, `/pipeline-runs` 228,
`/automation/scripts` 306, `/projects` ~360.

## Status: waves 1 and 2 done and **verified**, nothing committed

Branch `feat/coder-cli-permission-mode`. Gates on the current tree: `pnpm check` **exit 0,
0 errors**; `npx eslint src` **0 errors**. `pnpm lint` is **exit 1** solely on
`.claude/handoff.md` failing `prettier --check` — that is another session's file and is red
at `HEAD` too. Leave it.

### Landed

- **Global title bar** — `src/lib/components/layout/app-shell.svelte`: 56px → **40px**;
  icon buttons `h-8` → `h-7`. The Home button was deleted (the breadcrumb already renders a
  Home crumb). `--header-height` turned out to be dead — declared in `src/app.css`, read by
  nothing — and was removed along with the `headerHeight` key in
  `src/lib/config/layout-breakpoints.ts`.
- **`src/lib/components/shell/page-header.svelte`** — 80px → **36px**, one row: title,
  badge, then `|` and the description truncated, both `hidden sm:`.
- **`src/lib/components/shell/page-stats.svelte`** — four Cards → one **36px** inline strip,
  `tabular-nums`, `description` moved to the `title` attribute.
- **17 double-paddings stripped** — pages wrapping content in `container mx-auto p-6`
  _inside_ `PageContainer`'s own `py-4 md:py-6`. `container` also capped width on wide
  windows. Readable-width caps (`mx-auto max-w-4xl`) were deliberately kept on the form
  pages.
- **32 pages rolled onto the shared components** — `PageStats` had exactly **one** caller
  and `PageHeader` was hand-rolled as `text-3xl` on 19 pages, which is why fixing the shared
  components alone barely moved most routes.
- **Dashboard** (`src/routes/+page.svelte`) — greeting and Quick Actions deleted (all 7
  tiles duplicated sidebar entries); first content row **637 → 124px**.
- **13 stretched tab bars** — `TabsList class="w-full"`/`grid-cols-N` defeated the vendored
  `w-fit`; three labels were spanning 1172px.
- **Settings routes collapsed** — `/settings/frameworks-languages` (+ three `?tab=`
  variants) and `/settings/ai` retired behind 307 redirects;
  `SettingsNavigation.svelte` **−212 lines**; 16 pages stopped titling themselves twice.
- **`FlyEnvStyleDashboard.svelte` deleted** — 1049 lines, zero references outside its own
  barrel export in `src/lib/domains/sdk/index.ts`.

### Measured result

| route                                 | before | after   |
| ------------------------------------- | ------ | ------- |
| `/`                                   | 637    | **124** |
| `/deployments`                        | 212    | **80**  |
| `/credentials`                        | 308    | **204** |
| `/sdk`                                | 268    | **140** |
| `/settings/general` (first real card) | 286    | **202** |

UX-001/002/003/004/015 are `status: verified` in `.claude/qa/issues/` by a tester that did
not write them.

## What is left

A session is already running this: **"UX wave 3: cloud/SDK routes, right rails, splits"**
(`local_d840cda4-35b4-41bf-9f58-7d5d10228bfe`). Check its state before duplicating work.

The board is the brief: `.claude/qa/issues/UX-000-index.md` ranks all 19 findings by
win ÷ effort. Outstanding, in order:

- **UX-013** (approved) Cloud 20 route files → 4. `daemonsets/+page.svelte` and
  `statefulsets/+page.svelte` differ by 87 lines of ~200, all of it the resource name. Target
  `/cloud/[resource]` + descriptors in `cloud/core/resourceDescriptors.ts`, redirects from
  every old path. Largest change on the board; walk every redirect by hand.
- **UX-014** (approved) `/sdk` renders managers/services/versions inline _and_ as separate
  routes. ~500 lines.
- **UX-005** settings nav, **UX-006** log viewer, **UX-014** SDK dedup and **UX-016** forms
  are **done** in wave 3 (2026-11-07) but **not yet tested** — that session is deliberately
  holding its tester until `/settings` and `/sdk` stop being edited, so it does not measure a
  half-written tree.
- **UX-019** was already fixed upstream by another session.
- Still open: **UX-013** cloud routes, **UX-007** blocks, **UX-009** audit rail, **UX-012**
  tasks, and the **UX-018** splits of `SDKSidebar` and `IdeSettings`. Also **UX-008**
  (`/projects` 184 rows / 11731px, unvirtualised), **UX-010**, **UX-011**.

## Decisions already made, and why

- **Do not add a right sidebar to `shell-sidebar-layout.svelte`.** One was built and then
  **reverted**: `ResponsivePanel` (`$lib/components/shell`, `side="right"`, inline on
  desktop, Sheet below md) already does this and is in use at
  `src/routes/ai/chat/+page.svelte:612` and `CoderPanel.svelte:829`. Use it for the UX-009
  / 010 / 011 / 012 rails. `/projects/audit` is the strongest case — its keyboard triage
  loop shows **zero** rows at 1024x600.
- **`SDKSidebar.svelte` (954 lines) is the best split**: lines 602-760 and 766-954 are two
  copies of the same sidebar (icon-only and expanded). One tree plus
  `group-data-[collapsible=icon]:*`, as `app-shell.svelte` already does, **removes** ~400
  lines rather than moving them.
- **Four big files are to be left alone** despite line count: `ProjectAudit` 960 (one
  keyboard workflow — fix its layout, not its size), `CoderPanel` 852 (already composed),
  `sdk/ai/ollama` 880, `cloud/workloads/pods/[pod]` 910.
- **`PageHeader` takes `badgeVariant`.** Wave 2 moved three cloud detail pages onto it and
  silently dropped `variant={x.status === 'running' ? 'default' : 'secondary'}`, so running
  and failed looked identical. Fixed; keep passing it.

## Things that bit us

- **Several other Claude sessions edit this same checkout concurrently.** A Svelte 4→5
  migration plus a new `portal/no-legacy-syntax` eslint rule landed mid-session from
  elsewhere (`ModelCard`, `ModelList`, `ProgressIndicator`, `Workflow*`, `CommandPalette`,
  `SplitPaneContainer`, `eslint.config.js`, `vite.config.ts`). **Re-read any file
  immediately before editing it, never revert changes you did not make, and commit by path.**
  One concrete casualty: a `TabsList` fix was eaten when another agent deleted the file it
  lived in.
- **`.claude/qa/scripts/app.mjs` no longer works.** Playwright `connectOverCDP` throws
  `Assertion error … "type": "shared_worker"` every time — not transient. Replacement driver:
  `.claude/qa/scripts/cdp.mjs` (talks to the page target's `webSocketDebuggerUrl` directly).
  Every older QA script depends on the broken one.
- **A raw `page.goto` reboots the Tauri app** and can land on a restored route. Navigate by
  injecting an anchor and clicking it, so it goes through the SvelteKit client router.
- **`getByRole('button', {name})` is unsafe here** — containers with `role="button"` absorb
  their children's accessible names, so the query hits the wrapper and the click silently
  no-ops. Use `locator('button').filter({hasText})` and assert `tagName === 'BUTTON'`.
- **Never click by pixel coordinates.** A layout change silently reroutes every click.
- **The review's "16 → 12" settings arithmetic was wrong** (15 pages, not 16). Check a
  finding's numbers before quoting them.

## Also delivered: two app instances can now run side by side

`node scripts/dev-instance.mjs 2` → vite :1421, CDP :9223, its own database. Instance 1
(:1420/:9222) is untouched, as is existing data. Use instance 2 for testing so the owner
keeps instance 1; it also boots with an empty DB, so it is the safe place for destructive
flows. Five things must differ per instance and **four are invisible until you run it** —
dev port (a `predev` `kill-port 1420` used to kill instance 1), data dir, CDP port, cargo
target dir (Windows will not relink a running `.exe`), and the WebView2 user-data folder
(processes sharing one must pass identical browser arguments). Details in
`scripts/dev-instance.mjs` and the `running-two-app-instances` memory.

## Open questions for the owner

1. **Commit?** Nothing on this branch is committed. Commit our files **by path** — the tree
   also holds another session's migration.
2. **The Docker-down path is unconfirmed.** `DockerStatusBanner.svelte:53` went
   `{#if status}` → `{#if status && !status.running}`. The body is byte-identical so Start
   Docker / Retry should be intact, and it was verified by code read, but the tester
   declined to stop Docker (19 containers running on the owner's machine) — rightly.
   Confirm live next time Docker is legitimately down, on both `/deployments` and the
   terminal's SessionLauncher.
3. **UX-019** — the `Settings` button in the `/sdk` header does nothing;
   `src/routes/sdk/+page.svelte:267` has no `onclick`. Pre-existing, identical at `HEAD`.
