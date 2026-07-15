# Actions

Portal runs project automation through a single **Actions** model.

## Concepts

| Term | Meaning |
|------|---------|
| **Action** | One runnable unit (`install`, `build`, `github:…`, `n8n:…`) |
| **Workflow** | Named DAG/list of actions (`ci`, custom file workflows) |
| **Runner** | `local` (shell), `github` (Actions dispatch), or `n8n` |

## DX

```ts
import { actions } from "$lib/domains/actions";

// Single action
await actions.forProject(project).run("build");

// List
await actions.forProject(project).run(["install", "test", "build"]);

// Workflow
await actions.forProject(project).run("ci");

// Any folder
await actions.forDirectory("D:/my-app").run("install");
```

## Definition priority (highest wins)

1. Optional `.portal/pipeline.yml` (or `.yaml` / `.json`) in the project root  
2. Custom actions saved in Portal (when present)  
3. Smart defaults from project metadata (framework, package manager, `build_command` / `start_command` / `test_command`)

### Example `.portal/pipeline.yml`

```yaml
version: 1
actions:
  migrate:
    name: Migrate DB
    run: pnpm db:migrate
    category: utility
workflows:
  release:
    name: Release
    steps:
      - action: install
      - action: test
        needs: [install]
      - action: build
        needs: [test]
      - action: migrate
        needs: [build]
```

## Where to look in the UI

- Project → **Actions** tab — catalog, multi-select run, history, GitHub runs  
- **Action Runs** in the nav — global local execution history  
- `/automation` Quick Run — cwd-bound block runs (legacy-compatible)

Legacy `/projects/:id/pipelines` redirects to the Actions tab. Auto-provisioned Install/Dev/Build DB pipelines are no longer created on project open.
