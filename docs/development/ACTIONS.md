# Actions

Portal runs project automation through a single **Actions** model.

## Concepts

| Term         | Meaning                                                     |
| ------------ | ----------------------------------------------------------- |
| **Action**   | One runnable unit (`install`, `build`, `github:…`, `n8n:…`) |
| **Workflow** | Named DAG/list of actions (`ci`, custom file workflows)     |
| **Runner**   | `local` (shell), `github` (Actions dispatch), or `n8n`      |

## DX

```ts
import { actions } from '$lib/domains/actions';

// Single action
await actions.forProject(project).run('build');

// List
await actions.forProject(project).run(['install', 'test', 'build']);

// Workflow
await actions.forProject(project).run('ci');

// Any folder
await actions.forDirectory('D:/my-app').run('install');
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

## Releases

A `release:` block in `.portal/pipeline.yml` marks two workflows as the project's
release: the one that **ships** and the one that **rehearses** it. Steps may set
`cwd` (relative to the project) and `env`; `release.env` applies to every step and
a step's own `env` wins. Variables (`RELEASE_DRY_RUN`, `BUMP`) reach steps as env.

```yaml
release:
  kind: flutter-android # label only; nothing branches on it
  ship: ship # workflow that really deploys
  rehearse: ship-dry # required for rehearsing; it never runs `ship`
  bump: minor # exposed as ${BUMP} / $BUMP
  env: { PORTAL_PROGRESS: plain }
actions:
  deploy: { run: npm run deploy, cwd: server }
```

Project -> Actions shows a **Release** card (Rehearse / Ship). Commands, all in
`src-tauri/src/domains/projects/pipelines/commands.rs`:

| command                            | reachable over the daemon | does                                     |
| ---------------------------------- | ------------------------- | ---------------------------------------- |
| `release_describe {projectId}`     | yes                       | the `release:` block and resolved steps  |
| `release_list {projectId, limit?}` | yes                       | recent release executions                |
| `release_rehearse {projectId}`     | yes                       | runs `rehearse` with `RELEASE_DRY_RUN=1` |
| `release_ship {projectId}`         | **no** (`local_only`)     | runs `ship` with `RELEASE_DRY_RUN=0`     |

Watch and stop with the existing `get_pipeline_execution` and
`cancel_pipeline_execution`. Runs are ordinary pipeline executions (the file's
workflow is written to a `pipelines` row with `preset_key = "release:<workflow>"`
on each run). Cancel kills the process tree on Windows, but not detached Gradle
daemons, and a Cloud Run deploy already submitted may still finish. Closing the
window ends running releases (not yet fixed).

## Where to look in the UI

- Project → **Actions** tab — catalog, multi-select run, history, GitHub runs
- **Action Runs** in the nav — global local execution history
- `/automation` Quick Run — cwd-bound block runs (legacy-compatible)

Legacy `/projects/:id/pipelines` redirects to the Actions tab. Auto-provisioned Install/Dev/Build DB pipelines are no longer created on project open.
