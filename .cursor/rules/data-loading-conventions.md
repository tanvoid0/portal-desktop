# Data Loading Conventions (Portal Desktop)

Goal: minimize network/Tauri requests by loading only the data each page truly needs, while keeping shared badge/card data consistent via a lightweight TTL dashboard overview.

## Core rules
- Never trigger backend loads at module import time (avoid side effects like `loadTasks()` at the bottom of a store file). Route/page components must control when loading happens.
- For route-specific data, load in the page/component lifecycle (`onMount` / `$effect` keyed to the component being rendered). This guarantees “open the page -> load its data”.
- For global/shared UI (nav badges / home overview), do not depend on full project/task stores. Use a dedicated lightweight “dashboard overview” endpoint and a TTL-cached store loaded from `src/routes/+layout.svelte`.
- When data changes due to mutations (create/update/delete/toggle), invalidate the dashboard overview cache and refresh it (or let TTL expire) so the home cards + badges update consistently without forcing full page reloads.

## Implementation pattern (what future PRs should look like)
- Backend: add/extend lightweight aggregated commands/endpoints for dashboards; do not fetch full lists when counts/summaries are enough (e.g. counts by status instead of `get_all_projects` or `get_tasks`).
- Frontend:
  - Load shared dashboard summary via `dashboardStore.load()` in layout (`+layout.svelte`) so badges can stay populated.
  - Update shared dashboard summary after mutations via `dashboardStore.invalidate()` followed by `dashboardStore.refresh(true)` (or trigger a refresh on next access).
  - Keep route components isolated: `/tasks` loads tasks; `/sdk/manager` loads only SDK managers; Home (`/`) loads only the dashboard overview.

## Routing / gating requirements
- `+layout.svelte` is shared across routes, so it must avoid calling “expensive-but-shared” loaders on pages that do not need them.
- Concretely:
  - For `/sdk/*` routes, avoid loading the project/task dashboard overview in `+layout.svelte` unless that data is explicitly required for the SDK UI contract.
  - Prefer: `if (!isSdkPage) void dashboardStore.load();` (or an equivalent conditional keyed to the UI that consumes the data).

## TTL + refresh semantics
- Use TTL caching for shared badge data so navigation does not constantly refetch.
- Only force refresh (`refresh(true)`) after mutations that affect badge/home numbers.
- If multiple mutations happen in quick succession, consider batching (invalidate once, refresh once) to avoid redundant overview requests.

## References (current implementation)
- Backend command: `get_dashboard_overview` in `src-tauri/src/domains/dashboard/commands.rs`
- Dashboard frontend cache/store:
  - `src/lib/domains/dashboard/services/dashboardService.ts`
  - `src/lib/domains/dashboard/stores/dashboardStore.ts`
- Lazy task loading fix:
  - removed module-level auto-load in `src/lib/domains/tasks/stores/taskStore.ts`
  - added explicit `taskActions.loadTasks()` in `src/lib/domains/tasks/components/TaskManager.svelte`
