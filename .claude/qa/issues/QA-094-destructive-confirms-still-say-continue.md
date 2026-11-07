---
id: QA-094
title: Destructive confirms still say "Continue" on 28 call sites — IDE, mapping and framework delete among them
status: open
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

<!-- coder -->

## Verification

<!-- qa-tester -->
