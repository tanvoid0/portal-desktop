---
id: UX-016
title: Create/edit pages disagree on whether they have a heading, and every form section restates its own title
status: verified
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

2026-11-07 — tauri, dev instance 2. Ran `node .claude/qa/scripts/ux016-verify.mjs`,
`ux016-verify2.mjs`, `ux016-verify3.mjs`.

Exactly one `h1`, one `<main>` and a short run-up to the first field on every route:

| route | h1 | first field y |
| --- | --- | --- |
| `/scripts/new` | New script | 190 |
| `/projects/create` | New project | 218 |
| `/projects/edit/1` | Edit project | 218 |
| `/deployments/new` | New deployment | 186 |
| `/tasks/create` | New task | 158 |
| `/documents/create` | New document | 158 |

(The only other headings on the page are the app sidebar's own "NAVIGATION" / "TOOLS".)

`/scripts/new`: the header carries `Import JSON`, `Cancel`, `Save Script`. Import JSON opens the
"Import from JSON" Dialog with both paths — **Load from File** and a paste textarea:

- pasting a valid script JSON filled Name with "QA Pasted Script" and closed the dialog;
- `Load from File` with `script-good.json` filled Name with "QA Imported Script";
- a JSON missing `command` showed "Missing required field: command" inline and did not import;
- `/scripts/new?import=true` auto-opened the dialog.

`/deployments/new`: header holds `Cancel` and `Deploy (build & run)`; `?projectPath=D:/projects/qa-demo`
prefilled the path input; `Cancel` returned to `/deployments`. I did **not** press Deploy — it
builds and runs a container, which is a side effect outside the app.

Not covered: `/tasks/<id>/edit` and the `/documents/<id>` "Edit Document" card title — instance 2's
database has no documents (`a document: null`), and I did not create one to avoid leaving fixtures
on a route I was not asked to sweep.

The submit/cancel buttons staying in the form footer on the projects, tasks and documents routes is
recorded as deliberate and was not filed.

Shots: `.claude/qa/shots/UX-016-after-scripts-new-paste-import.png`,
`.claude/qa/shots/UX-016-after-scripts-new-file-import.png`,
`.claude/qa/shots/UX-016-after-deployments-new.png`.

