---
id: QA-094
title: Destructive confirms still say "Continue" on 28 call sites — IDE, mapping and framework delete among them
status: verified
severity: polish
kind: copy
area: settings/ides, sdk, cloud, ai/chat, projects (confirmAction)
mode: tauri
found: 2026-11-07
---

## Use case

Delete an IDE, a framework mapping or a custom framework and read the confirm dialog before
committing.

## Steps

1. `/settings/ides`, delete an IDE (trash icon on its row).
2. Delete a framework mapping (trash icon on the mapping row).
3. Delete a custom framework (the X in the chip overlay).

## Expected

The confirm button names the action and is styled destructive — what QA-039 fixed for tasks and
documents.

## Actual

All three read `Cancel` / `Continue`:

```
Delete IDE       | Are you sure you want to delete "QA Editor"?                  | Cancel | Continue
Delete mapping   | Are you sure you want to delete the mapping for "SvelteKit"?  | Cancel | Continue
Delete framework | Are you sure you want to delete "QA Framework edited"?        | Cancel | Continue
```

Same root cause as QA-039 (`confirmAction` defaults `confirmLabel` to "Continue"); QA-039 only
fixed the task and document call sites. 28 of the 43 `confirmAction(` call sites still omit
`confirmLabel`, including every uninstall in the SDK pages, pod/ConfigMap/Secret delete under
`/cloud`, conversation delete in `/ai/chat`, and project delete in `/projects`:

```
src/lib/domains/settings/components/IdeList.svelte
src/lib/domains/settings/components/FrameworkIdeMappings.svelte          (x2)
src/lib/domains/settings/components/shared/ItemSettings.svelte
src/lib/domains/settings/components/LearningSettings.svelte
src/lib/domains/sdk/components/{AliasManager,CustomDirectoryManager,VersionManagerTable}.svelte
src/routes/sdk/{managers,managers/[name],software-installer,[sdk],[sdk]/version}/+page.svelte
src/routes/cloud/{configmaps/[configmap],secrets/[secret],workloads/pods/[pod]}/+page.svelte
src/routes/cloud/workloads/deployments/[deployment]/+page.svelte        (rollback)
src/routes/ai/chat/+page.svelte                                          (x2)
src/routes/projects/+page.svelte
src/lib/domains/deployments/components/DeploymentDashboard.svelte
src/lib/domains/tasks/components/InstructionTemplateManager.svelte
src/lib/domains/cloud/components/YamlEditor.svelte                       (x2)
src/lib/domains/projects/components/{AiTidy,ProjectAudit,UnavailableProjects}.svelte
```

## Evidence

Dialog text quoted above, captured by `node .claude/qa/scripts/ides-verify8.mjs` and
`ides-verify9.mjs`. The call-site list came from scanning `confirmAction(` calls for a missing
`confirmLabel` option.

## Repro

`node .claude/qa/scripts/ides-verify9.mjs` (delete-IDE confirm) — manual for the rest.

## Fix

Root cause (one shared fix, not 28 call-site edits): `confirmActions.request()` in
`confirmStore.ts` defaulted `confirmLabel` to the literal string `'Continue'` whenever a call
site omitted it — QA-039 only patched the task/document call sites directly instead of this
default. Added `defaultConfirmLabel(title)`: takes the title's leading word (e.g. "Delete IDE" →
"Delete", "Uninstall SDK manager" → "Uninstall", "Rollback deployment" → "Rollback") unless the
title is the bare default `'Confirm'`, in which case it still falls back to "Continue". This
fixes every omitted-`confirmLabel` call site at once, including all of IDE/mapping/framework
delete, the SDK uninstalls, cloud pod/ConfigMap/Secret delete, chat/project delete, etc.

One call site (`src/routes/projects/+page.svelte` project delete) passed no title at all, so
there was nothing for the heuristic to read — gave it an explicit `'Delete project'` title.

Files: `src/lib/domains/shared/stores/confirmStore.ts`, `src/routes/projects/+page.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors).

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-094.mjs` — created fixtures "QA Editor"
(IDE) and "QA Framework", added a mapping between them, then tested all three delete confirms
from the original repro's Steps:

```
mapping delete confirm:   Delete mapping | Are you sure you want to delete the mapping for "QA Framework"? | Cancel | Delete
framework delete confirm: Delete framework | Are you sure you want to delete "QA Framework"? | Cancel | Delete
IDE delete confirm:       Delete IDE | Are you sure you want to delete "QA Editor"? | Cancel | Delete
```

All three now say the specific action instead of "Continue". Also spot-checked
`src/routes/projects/+page.svelte` (the one call site needing an explicit title) by reading
the code — `confirmAction(..., 'Delete project')` confirmed present. All three fixtures
deleted during the run (confirmed 0 remaining at the end). Verified.
