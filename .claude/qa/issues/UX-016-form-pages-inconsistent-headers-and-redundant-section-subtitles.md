---
id: UX-016
title: Create/edit pages disagree on whether they have a heading, and every form section restates its own title
status: fixed
severity: minor
kind: ux
area: forms (*/create, */new, */edit)
mode: tauri
found: 2026-11-06
---

## Use case

Create a script, a task, a document, a deployment, a project.

## Measured (1280x800)

| route | h1 | chrome before first field |
| --- | --- | --- |
| `/scripts/new` | `<h1 text-2xl>Create New Script` | 108px |
| `/projects/create` | `Create New Project` | 80px |
| `/deployments/new` | `Create New Deployment` | 140px |
| `/tasks/create` | **none** | 24px |
| `/documents/create` | **none** | 24px |

`/tasks/create` and `/documents/create` render no `<h1>` at all — the page is a bare
`<div>` around `TaskForm` / `DocumentEditor`
([src/routes/tasks/create/+page.svelte:20-22](../../../src/routes/tasks/create/+page.svelte#L20),
[src/routes/documents/create/+page.svelte](../../../src/routes/documents/create/+page.svelte)).
The three that do have one hand-roll it rather than using `PageHeader`, so the back arrow,
the title and the save button are laid out three different ways across five forms.

## Section subtitles say nothing

`/scripts/new` ([src/routes/scripts/new/+page.svelte](../../../src/routes/scripts/new/+page.svelte)):

| line | CardTitle | CardDescription |
| --- | --- | --- |
| :259 | Basic Information | "Script name, description, and categorization" |
| :320 | Command Configuration | "Define the command or script to execute" |
| :360 | Parameters | "Define configurable parameters for your script" |
| :472 | (preview) | "JSON representation of your script" |

Each `CardDescription` is ~40px and restates its own title. Four of them is 160px on a form
that is already 1183px tall in a 760px viewport.

## Import from JSON sits above the form

`/scripts/new:207-212` — a collapsed `Import from JSON` accordion occupies the **top 120px**
of the page, above `Name`. It is the rarely-used path; the common path (type a name) starts
below it.

## Fix

1. Every `*/create|new|edit` route uses `PageHeader` with the primary action in its
   `actions` snippet — including `/tasks/create` and `/documents/create`, which get
   `title="New task"` / `title="New document"`. One shape for all of them.
2. Delete the four `CardDescription`s on `/scripts/new`, and the equivalents on
   `/deployments/new` and `/projects/create`. Keep a description only where it states a
   constraint the title does not (e.g. a format or a limit).
3. Move `Import from JSON` out of the page body: a `variant="outline" size="sm"` **Import
   JSON** button in the PageHeader actions that opens a `Dialog`. −120px and the form starts
   at the first field.

## Evidence

`.claude/qa/shots/ux/ux-scripts-new.png`, `.claude/qa/shots/ux/ux-projects-create.png`,
`.claude/qa/shots/ux/ux-tasks-create.png`, `.claude/qa/shots/ux/ux-documents-create.png`,
`.claude/qa/shots/ux/ux-deployments-new.png`.

## Repro

`node .claude/qa/scripts/app.mjs scripts/new ux-scripts-new`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->
