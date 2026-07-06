# Data Loading Conventions (Portal Desktop)

Goal: minimize network/Tauri requests by loading only the data each page truly needs, while keeping shared badge/card data consistent via a lightweight TTL dashboard overview.

## Core rules

- Never trigger backend loads at module import time (avoid side effects like `loadTasks()` at the bottom of a store file). Route/page components must control when loading happens.
- For route-specific data, load in the page/component lifecycle (`onMount` / `$effect` keyed to the component being rendered). This guarantees “open the page -> load its data”.
- For global/shared UI (nav badges / home overview), do not depend on full project/task stores. Use a dedicated lightweight “dashboard overview” endpoint and a TTL-cached store loaded from `src/routes/+layout.svelte`.
- When data changes due to mutations (create/update/delete/toggle), invalidate the dashboard overview cache and refresh it (or let TTL expire) so the home cards + badges update consistently without forcing full page reloads.

## TanStack Query (server state)

For list/detail data owned by the Rust backend, prefer **TanStack Query** over manual store + `cache.ts` duplication:

- **Provider:** `QueryProvider` in `src/routes/+layout.svelte` (singleton `queryClient` in `src/lib/domains/shared/query/queryClient.ts`)
- **Query keys:** `src/lib/domains/shared/query/keys.ts`
- **Dashboard invalidation:** `invalidateDashboardOverview(queryClient)` after mutations that affect home cards or nav badges
- **Projects list:** `createProjectsQuery()` in `src/lib/domains/projects/queries/projectQueries.ts`

Legacy `cache.setList("projects", …)` remains for domains not yet migrated — do not add new `writable()` stores for server data.

**Tasks:** `createTasksQuery()` / `createTaskQuery()` in `src/lib/domains/tasks/queries/`; `TaskManager` syncs query data via `taskUi.setTasks()`; mutations in `taskActions` (`taskStore.ts`); UI session state in `state/taskUi.svelte.ts`.

## Implementation pattern (what future PRs should look like)

- Backend: add/extend lightweight aggregated commands/endpoints for dashboards; do not fetch full lists when counts/summaries are enough (e.g. counts by status instead of `get_all_projects` or `get_tasks`).
- Frontend:
  - Load shared dashboard summary via `createDashboardOverviewQuery()` in layout (`app-shell.svelte`); skip on `/sdk/*` via `enabled: false`.
  - After mutations, call `invalidateDashboardOverview(queryClient)` (and domain-specific invalidation, e.g. `invalidateProjectsList`).
  - Keep route components isolated: `/tasks` loads tasks; `/sdk/manager` loads only SDK managers; Home (`/`) uses the shared dashboard query cache.

## Routing / gating requirements

- `+layout.svelte` is shared across routes, so it must avoid calling “expensive-but-shared” loaders on pages that do not need them.
- Concretely:
  - For `/sdk/*` routes, the dashboard overview query uses `enabled: false` in `app-shell.svelte`.
  - Prefer Query `enabled` flags over imperative `load()` calls in layout.

## TTL + refresh semantics

- Use TTL caching for shared badge data so navigation does not constantly refetch.
- Only force refresh (`refresh(true)`) after mutations that affect badge/home numbers.
- If multiple mutations happen in quick succession, consider batching (invalidate once, refresh once) to avoid redundant overview requests.

## List cache semantics (avoid stale empty UI)

The in-memory cache returns `null` on miss/expiry. **Never use `if (cached)` after `cache.get()` for arrays** — `[]` is truthy, so a stale empty list skips the backend until TTL expires or a mutation calls `cache.delete()`.

Use the list helpers in `src/lib/domains/shared/services/cache.ts`:

- **`cache.getList(key)`** — returns `null` on miss/expiry **or** when the cached list is empty (forces a fresh fetch).
- **`cache.setList(key, items, ttl)`** — stores non-empty lists; empty results delete the key instead of poisoning the cache.
- Pass `{ allowEmpty: true }` / `{ cacheEmpty: true }` only when a verified empty list should be cached (rare).

Also:

- Prefer **`cached !== null`** over **`if (cached)`** when empty objects/arrays are valid stored values but you need an explicit miss check.
- Do not cache error fallbacks or pre-ready startup responses (e.g. dashboard `EMPTY_OVERVIEW` before the backend is available) unless intentional.
- After create/update/delete mutations, **`cache.delete(key)`** for the affected list (already done for projects).

Other call sites to audit: `learningService.getSuggestions`, `projectService.getStats`, `dashboardService.getDashboardOverview` (object cache — lower risk, but avoid priming all-zero fallbacks after failed fetches).

## References (current implementation)

- Backend command: `get_dashboard_overview` in `src-tauri/src/domains/dashboard/commands.rs`
- Dashboard frontend:
  - `src/lib/domains/dashboard/queries/dashboardQueries.ts` (TanStack Query)
  - `src/lib/domains/dashboard/services/dashboardService.ts`
- Tasks frontend:
  - `src/lib/domains/tasks/queries/taskQueries.ts`
  - `src/lib/domains/tasks/api/taskApi.ts`
  - `src/lib/domains/tasks/state/taskUi.svelte.ts` (UI session state — filters, selection, time tracking)
  - `src/lib/domains/tasks/stores/taskStore.ts` (mutations only)
- Lazy task loading fix:
  - removed module-level auto-load in `src/lib/domains/tasks/stores/taskStore.ts`
  - added explicit `taskActions.loadTasks()` in `src/lib/domains/tasks/components/TaskManager.svelte`
