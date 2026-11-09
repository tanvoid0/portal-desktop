---
id: UX-011
title: Project detail stacks three header rows then runs one 1172px column of fact cards
status: verified
severity: major
kind: ux
area: projects/[id]
mode: tauri
found: 2026-11-06
---

## Use case

Open a project and run it, or check its git state.

## Measured (1280x800, /projects/1)

192px of chrome before the first tab panel, in four stacked rows:

| row | y | px | file |
| --- | --- | --- | --- |
| `← Back to Projects` ghost button | 80 | 50 | [src/routes/projects/[id]/+page.svelte:382-385](../../../src/routes/projects/[id]/+page.svelte#L382) |
| PageHeader: name + status badge + `Open in editor` `Open folder` `Terminal` `More` | 138 | 54 | `:392-454` |
| `ProjectSummary` — the absolute path + a copy button, alone on its row | 192 | 40 | [ProjectSummary.svelte](../../../src/lib/domains/projects/components/ProjectSummary.svelte) |
| tabs `Overview / Actions / Terminal`, forced `grid w-full grid-cols-3` | 247 | 48 | `:460` (also UX-004) |

The breadcrumb at the top of the window already reads `Home › Projects › Portal Desktop`, so
the "Back to Projects" button is a third way to do what the breadcrumb and the browser Back
button in the title bar already do.

Below that, the Overview tab is a single column of full-width cards in a **1172px** column,
each of which holds a few words:

- **How to run this** / "Commands that start, build and test this project." — 200px, body was
  one sentence of empty state.
- **What it's built with** / "The languages and tools this project uses." — 150px, and it
  renders the same information **twice**: a prose sentence ("A Svelte and Tauri project
  written in TypeScript and HTML and CSS and Rust, using pnpm.") and then the same six items
  as badges directly below it. The sentence is also ungrammatical for >2 items.
- **Git** / "Is your work saved and backed up?" — the card subtitles are questions aimed at a
  non-developer; this is a developer-first app.

## Expected

The facts about a project (path, stack, git, status) are reference material you want visible
while you work in the main column.

## Actual

They are stacked full-width down a single column so you scroll past them to reach anything
actionable.

## Fix

1. Delete the `← Back to Projects` button (`+page.svelte:382-385`) and `handleBack`. The
   breadcrumb and the title-bar Back button both already do it. −50px.
2. Fold `ProjectSummary`'s path row into the PageHeader as the `description` slot (monospace,
   truncated, with the copy button). −40px.
3. Give the page a right rail with `ResponsivePanel side="right" desktopClass="w-80"` holding
   `ProjectTechStack` and `ProjectGitCard`, unchanged — they already take `project` and
   `githubUrl` as props, so this is a move, not a rewrite. The main column keeps
   `ProjectCommands` and `ProjectModules`, which is where the clicking happens.
4. In `ProjectTechStack`, delete the prose sentence and keep the badges. Delete the
   `CardDescription` line from all three cards — "Is your work saved and backed up?" under a
   card titled **Git** adds nothing.

Result: first content row at ~y=100 instead of 232, and the stack/git facts stay on screen
while you use the Actions and Terminal tabs.

## Evidence

`.claude/qa/shots/ux/ux-projects-1.png`.

## Repro

`node .claude/qa/scripts/app.mjs projects/1 ux-project-detail`

## Fix

<!-- coder -->

Root cause: the page spent three stacked rows on navigation it did not need (a Back button the
breadcrumb already covers, a path strip on its own row) and then rendered reference facts as
full-width cards in the single content column.

- `src/routes/projects/[id]/+page.svelte` — deleted the `← Back to Projects` button, `handleBack`
  and the now-unused `ArrowLeft` import; the path strip (`ProjectSummary`, unchanged) now renders
  inside `PageHeader` via a new `subtitle` snippet, so the copy button survives; `ProjectTechStack`
  and `ProjectGitCard` moved into a `ResponsivePanel side="right" desktopClass="w-80" title="Details"`
  that sits **outside** the `Tabs`, so the stack and git facts stay on screen in Actions and
  Terminal; the main column keeps `ProjectCommands` (+ "Last opened") and the Apps tab's
  `ProjectModules`. Below `md` the rail is the primitive's Sheet, opened by a `md:hidden` info
  button in the header actions. The header's prose `description` gave up its place to the path and
  moved to the top of the rail as an "About" card (`const about = $derived(readableDescription(...))`,
  rendered only when non-empty — no card, no placeholder, when the project has no description).
- `src/lib/components/shell/page-header.svelte` — additive optional `subtitle?: Snippet`, rendered
  next to `description` in the same 36px row. No existing caller changes.
- `src/lib/domains/projects/components/ProjectTechStack.svelte` — deleted the duplicate prose
  sentence (and its `sentence`/`describeModules` machinery) and the `CardDescription`.
- `src/lib/domains/projects/components/ProjectGitCard.svelte` — deleted "Is your work saved and
  backed up?".
- `src/lib/domains/projects/components/ProjectCommands.svelte` — deleted "Commands that start,
  build and test this project.".

Chrome: 50 + 54 + 40 + 48 = 192px of header rows → 54 + 48 = 102px, and two of the three 24px
`space-y-6` gaps go with them (~138px reclaimed before the first tab panel). The fact cards are now
a 320px rail instead of a 1172px column.

UX-004 on this page: `TabsList` here already carried no `grid w-full grid-cols-3` (fixed in an
earlier pass); nothing left to drop.

Checks: `npx eslint src/routes/projects src/lib/domains/projects src/lib/components/shell` → 0
errors (pre-existing warnings only); `pnpm check` → 0 errors, no warning in a touched file. Not
verified in the running app — no app was listening on CDP 9222 and the brief forbade starting one.

## Verification

2026-11-09 — tauri, `/projects/4` (Devstrail). Single header row: name + `active` badge +
monospace truncated path with a copy icon + `Open in editor` / `Open folder` / `Terminal` /
`More`, tabs (`Overview / Actions / Env / Terminal`) directly below at y≈124 — no
`← Back to Projects` button anywhere (count 0). Right rail titled "Details" holds `What it's
built with` (badges only — `Next.js`, `Node.js`, `JavaScript`, `TypeScript`, `pnpm` — no
duplicate prose sentence) and `Git` (branch, uncommitted/unpushed/unbacked-up state, no "Is
your work saved and backed up?" subtitle). Main column holds `How to run this` (Start/Build/
Test/Build output rows). Matches the Fix description exactly. Screenshot:
`.claude/qa/shots/QA-UX011-after-1280x800.png` (clean — no QA-104 overlay on this load).
**Verified.**
