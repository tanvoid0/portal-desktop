# Svelte Maintainability Plan — Portal Desktop

## Overview

This document defines a phased plan to improve frontend maintainability **without migrating away from Svelte**. The goal is to reduce complexity, eliminate duplicated state, and establish consistent patterns across ~521 Svelte components and 28 legacy stores.

**Related docs:**

- [Svelte Patterns Review](./SVELTE_PATTERNS_REVIEW.md) — runes, lifecycle, component structure
- [Data Loading Conventions](../../.cursor/rules/data-loading-conventions.md) — lazy loading, dashboard TTL cache
- [Complete Project Blueprint](./COMPLETE_PROJECT_BLUEPRINT.md) — overall architecture

**Decision record:** A full React migration is **not recommended**. The pain points are app scale and mixed state patterns, not a wrong framework choice. See [Why not React?](#why-not-react) below.

---

## Current state (baseline)

| Metric | Approx. value |
|--------|---------------|
| Svelte components | ~521 |
| Route pages | ~88 |
| shadcn-svelte UI primitives | ~264 |
| Legacy `writable()` stores | 28 |
| `.svelte.ts` state modules | 3 |
| TanStack Query usage | None |

### What works well

- **Svelte 5 runes** are adopted in components (`$state`, `$derived`, `$props`, `$effect`)
- **Domain-driven structure** — `src/lib/domains/*` with services, types, components
- **Plain TypeScript services** — e.g. `projectService.ts` wraps Tauri invoke calls
- **Rust backend** — performance-critical work stays in Tauri; UI framework is not the bottleneck
- **Dashboard TTL cache** — `dashboardStore` + `dashboardService` is a good reference pattern

### What causes maintainability pain

1. **Two state mental models** — Svelte 5 runes in components + Svelte 4 `writable`/`derived` stores globally
2. **Triple caching** — store state + `cache.ts` in services + manual loading flags in components
3. **Service ↔ store coupling** — services mutate stores directly (e.g. `projectStore.setLoading(true)`)
4. **Domain breadth** — terminal, K8s, SDK, AI, pipelines, deployments in one shell
5. **Store sprawl in hot domains** — terminal has 7 separate stores; `taskStore.ts` is ~720 lines
6. **Iteration residue** — archived terminal variants under `components/archived/`

---

## Target architecture

Every domain should converge on three layers:

```
┌─────────────────────────────────────────────────────────┐
│  UI Layer — +page.svelte, domain components             │
│  Renders data, handles events, local $state for forms     │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│  Data Layer                                              │
│  • TanStack Query — server/cache state (lists, details)  │
│  • .svelte.ts classes — UI/session state (tabs, filters)│
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│  Core Layer — framework agnostic                         │
│  • *Service.ts / *Api.ts — invoke, mapping, side effects │
│  • types/ — shared interfaces                            │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
                    Tauri invokeClient → Rust
```

### Layer responsibilities

| Layer | Owns | Does not own |
|-------|------|--------------|
| `*Service.ts` / `*Api.ts` | Tauri calls, DTO mapping, logging, learning hooks | Loading spinners, `$state`, route params |
| TanStack Query | Fetch, cache, stale time, retry, invalidation | Tab selection, form drafts, sidebar open |
| `.svelte.ts` class | Ephemeral UI state (tabs, filters, selection) | Duplicating backend lists with fake optimistic IDs |
| Components | Render + wire events | Direct `invokeClient` calls |

### Reference pattern: `.svelte.ts` state class

The sidebar context is the template for replacing `writable()` stores:

```typescript
// src/lib/components/ui/sidebar/context.svelte.ts
class SidebarState {
  open = $derived.by(() => this.props.open());
  openMobile = $state(false);

  constructor(props: SidebarStateProps) {
    $effect(() => {
      if (this.#isMobile.current) {
        this.openMobile = this.open;
      }
    });
  }
}
```

Use `setContext` / `getContext` (or a module singleton for app-wide UI state) to expose these classes.

---

## State taxonomy

Split all application state into three buckets. **Do not mix them in one store.**

### 1. Server state

Data owned by the Rust backend: project lists, tasks, K8s resources, SDK versions.

**Tool:** `@tanstack/svelte-query`

```typescript
createQuery(() => ({
  queryKey: queryKeys.projects.all,
  queryFn: () => projectApi.fetchAll(),
}));
```

### 2. Client UI state

Ephemeral, never persisted to backend: active terminal tab, selected table rows, command palette open, breadcrumb trail.

**Tool:** `.svelte.ts` classes or component-local `$state`

### 3. Cross-cutting infra

Theme, toasts, sidebar layout — migrate last; low churn.

**Tool:** Existing shared stores → `.svelte.ts` when touched

### Anti-patterns to eliminate

| Anti-pattern | Example | Fix |
|--------------|---------|-----|
| Service mutates store loading flags | `projectStore.setLoading(true)` in `loadProjects()` | Query handles `isPending` |
| Manual list cache + store | `cache.set("projects", ...)` + `projectStore.setProjects()` | Query cache only |
| Optimistic fake entities | `projectStore.addProject()` generating local IDs | Mutation returns backend entity |
| Module-level auto-load | `loadTasks()` at bottom of store file | Page/component triggers load |
| Direct invoke in components | `invokeClient.post(...)` in `+page.svelte` | Route through service/api layer |

---

## TanStack Query setup

### Install

```bash
pnpm add @tanstack/svelte-query
```

### Provider (layout)

Wire in `src/routes/+layout.svelte`:

```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,  // align with existing 5min project cache
      gcTime: 30 * 60 * 1000,
      retry: 1,
    },
  },
});
```

Wrap app content in `<QueryClientProvider client={queryClient}>`.

### Query keys (central registry)

Create `src/lib/domains/shared/query/keys.ts`:

```typescript
export const queryKeys = {
  dashboard: {
    overview: ['dashboard', 'overview'] as const,
  },
  projects: {
    all: ['projects'] as const,
    detail: (id: number) => ['projects', id] as const,
  },
  tasks: {
    all: ['tasks'] as const,
    detail: (id: string) => ['tasks', id] as const,
  },
  cloud: {
    resources: (type: string, namespace: string) =>
      ['cloud', type, namespace] as const,
  },
  sdk: {
    managers: ['sdk', 'managers'] as const,
    versions: (name: string) => ['sdk', name, 'versions'] as const,
  },
};
```

### Dashboard invalidation helper

Extract repeated invalidation from `projectService.ts` and `taskStore.ts`:

```typescript
// src/lib/domains/shared/query/invalidateDashboard.ts
import type { QueryClient } from '@tanstack/svelte-query';
import { queryKeys } from './keys';

export function invalidateDashboardOverview(queryClient: QueryClient): void {
  queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.overview });
}
```

Call after mutations that affect home cards or nav badges.

### What Query does NOT handle

- **Tauri event streams** (terminal output, live logs) — use event listeners → append to `$state`
- **WebSocket / SSE** — same as above
- **Form drafts** — component-local `$state` until submit

---

## Domain folder structure (target)

Each domain converges on:

```
src/lib/domains/{domain}/
├── types/
├── api/              # pure invoke + mapping (no store writes)
├── services/         # orchestration: logging, learning, side effects
├── queries/          # TanStack Query hooks + mutations
├── state/            # *.svelte.ts UI/session classes (optional)
├── components/
└── index.ts          # public API only
```

### Public API (`index.ts`)

Export only what other domains and routes need:

```typescript
// ✅ Export
export type { Project, CreateProjectRequest } from './types';
export { createProjectsQuery, createProjectMutation } from './queries/projectQueries';
export { projectUi } from './state/projectUi.svelte';

// ❌ Do not export
// projectApi (internal)
// legacy projectStore (remove after migration)
```

### Import rules (enforce in code review)

```typescript
// ✅ Page / component
import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';

// ❌ Page / component
import { invokeClient } from '$lib/utils/invokeClient';
import { writable } from 'svelte/store';
```

---

## Mutation checklist

After any create, update, or delete:

1. `queryClient.invalidateQueries()` for affected list/detail keys
2. `invalidateDashboardOverview(queryClient)` if home cards or nav badges change
3. Do not manually patch list arrays unless implementing documented optimistic UI

---

## Phased migration plan

Incremental PRs per route or domain. **Do not bulk-convert all 28 stores.**

### Phase 0 — Foundation (1–2 weeks)

| Task | Files / area |
|------|--------------|
| Install `@tanstack/svelte-query` | `package.json`, `+layout.svelte` |
| Add `queryKeys.ts`, `invalidateDashboard.ts` | `src/lib/domains/shared/query/` |
| Extend data-loading conventions | `.cursor/rules/data-loading-conventions.md` |
| Delete archived terminal components if unused | `terminal/components/archived/` |
| Consolidate duplicate toast modules | `src/lib/stores/toast.ts` vs `toastStore.ts` |
| Remove `projectStore.addProject()` fake-ID path | `projectStore.ts` |

**Exit criteria:** Query provider works; dashboard invalidation is centralized; no new `writable()` stores added.

---

### Phase 1 — Projects pilot (2–3 weeks)

**Why first:** Small UI surface (~5 domain Svelte files), mature service layer, clear CRUD, already invalidates dashboard on mutations.

| Step | Action |
|------|--------|
| 1 | Create `projectApi.ts` — invoke-only, no store writes |
| 2 | Create `queries/projectQueries.ts` — list, detail, mutations |
| 3 | Refactor `routes/projects/+page.svelte` to use queries |
| 4 | Refactor create/edit routes |
| 5 | Shrink or remove `projectStore.ts` — keep `activeProjectId` in `projectUi.svelte.ts` if needed |
| 6 | Update `domains/projects/index.ts` exports |

**Reference files:**

- Service: `src/lib/domains/projects/services/projectService.ts`
- Store (legacy): `src/lib/domains/projects/stores/projectStore.ts`
- Page: `src/routes/projects/+page.svelte`

**Exit criteria:** Projects CRUD works with zero `writable` in the domain; no manual `cache.set("projects", ...)`.

This phase becomes the **copy-paste template** for all subsequent domains.

---

### Phase 2 — Dashboard + Tasks (2–3 weeks)

**Status (2026-06-12):** Complete. UI state in `taskUi.svelte.ts`; mutations in `taskStore.ts`.

#### Dashboard ✅

Replace `dashboardStore` with a query hook. In `+layout.svelte`, use `enabled: !isSdkPage` instead of conditional `void dashboardStore.load()`.

**Reference:** `src/lib/domains/dashboard/queries/dashboardQueries.ts`

#### Tasks ✅

| Module | Status |
|--------|--------|
| `queries/taskQueries.ts` | ✅ list + detail |
| `queries/invalidateTasks.ts` | ✅ cache invalidation |
| `api/taskApi.ts` | ✅ invoke-only fetch |
| `state/taskUi.svelte.ts` | ✅ filters, selection, time tracking, templates, saved views |
| `stores/taskStore.ts` | ✅ mutations only (`taskActions`) |
| `TaskManager.svelte` | ✅ loads via `createTasksQuery()` |
| Module-level auto-load | ✅ removed |

**Exit criteria:** `TaskManager.svelte` loads via query; no module-level auto-load; UI state in `taskUi.svelte.ts`. ✅

---

### Phase 3 — Settings, Credentials, Documents (2 weeks)

Form-heavy, low real-time complexity. Good ROI for daily development friction.

- Settings: single-record read/write → `useMutation`
- Credentials: explicit mutations only, no optimistic UI (security-sensitive)
- Documents: follow projects template

---

### Phase 4 — Cloud / K8s (3–4 weeks)

**Why later:** Many similar list/detail pages; namespace-scoped resources; connection state mixed with resource lists.

| Step | Action |
|------|--------|
| 1 | Generic `createResourceQuery(type, namespace)` factory |
| 2 | Migrate `cloudStore` connection fields to `cloudSession.svelte.ts` |
| 3 | Consolidate workload list pages using shared table components |
| 4 | Leave k8s keyboard hooks as plain TS — they are fine |

**Reference:** `src/lib/domains/cloud/stores/cloudStore.ts`, `src/routes/cloud/workloads/`

---

### Phase 5 — SDK Manager (3 weeks)

Complex sidebar UI (`SDKSidebar.svelte`), version lists, install status.

- Server data → Query
- Sidebar selection, expanded categories, install progress → `sdkUi.svelte.ts`
- Defer refactoring `FlyEnvStyleDashboard.svelte` until query layer exists

---

### Phase 6 — Terminal (4–6 weeks, last)

**Why last:** Real-time streaming, 7 stores, Tauri event listeners. Highest risk.

| Step | Action |
|------|--------|
| 1 | Merge `tabStore` + `sessionStore` + parts of `aiTerminalStore` → `terminalSession.svelte.ts` |
| 2 | Keep `commandHistoryStore` and `commandPaletteStore` separate (different lifecycles) |
| 3 | Streaming: Tauri events → append to `$state` in session class (not Query) |
| 4 | Remove archived V2 components once current path is confirmed |

**Stores to consolidate:**

- `aiTerminalStore.ts`
- `tabStore.ts`
- `sessionStore.ts`
- `terminalStore.ts`
- `commandHistoryStore.ts`
- `commandPaletteStore.ts`
- `terminalNotesStore.ts`

---

## Migration order summary

| Phase | Domain | Effort | Risk | Payoff |
|-------|--------|--------|------|--------|
| 0 | Query setup + conventions | 1–2 wk | Low | Foundation |
| 1 | **Projects** (pilot) | 2–3 wk | Low | Template for all domains |
| 2 | Dashboard + **Tasks** | 2–3 wk | Medium | Removes largest store mess |
| 3 | Settings, Credentials, Documents | 2 wk | Low | Daily friction ↓ |
| 4 | Cloud / K8s | 3–4 wk | Medium | Dedupe similar pages |
| 5 | SDK Manager | 3 wk | Medium | Sidebar complexity ↓ |
| 6 | **Terminal** | 4–6 wk | High | Do last |

**Estimated total:** 4–6 months alongside feature work, not a feature freeze.

---

## Component conventions

### Size limit

If a `.svelte` file exceeds ~300 lines:

- Extract `$derived` logic to `.svelte.ts`
- Split form sections into child components
- Keep the page as orchestration only

**Known offenders:** `ProjectForm.svelte` (587 lines), `+layout.svelte`

### Event handlers

Follow [Svelte Patterns Review](./SVELTE_PATTERNS_REVIEW.md):

```svelte
<Button onclick={handleRefresh}>Refresh</Button>
<Button onclick={() => goto('/projects')}>Go to Projects</Button>
```

### shadcn-svelte

Do **not** rewrite UI primitives under `src/lib/components/ui/`. They already use Svelte 5 runes correctly.

---

## Success metrics

Track monthly during migration:

| Metric | Baseline | Target @ 6 mo |
|--------|----------|---------------|
| `writable()` store files | 28 | ≤ 5 (theme, toast, terminal session) |
| Domains with Query layer | 0 | ≥ 8 |
| Components calling `invokeClient` directly | TBD | 0 outside services/api |
| Duplicate cache paths | several | 0 |
| Archived/dead UI files | 2+ | 0 |

---

## First PR scope (recommended)

A focused PR to prove the pattern without touching terminal or cloud:

1. Install `@tanstack/svelte-query` + provider in `+layout.svelte`
2. Add `src/lib/domains/shared/query/keys.ts`
3. Add `src/lib/domains/shared/query/invalidateDashboard.ts`
4. Migrate dashboard overview from `dashboardStore` to Query
5. Migrate `routes/projects/+page.svelte` list loading to Query
6. Remove store/cache writes from `projectService.loadProjects()` fetch path

**Estimated touch:** 8–12 files.

---

## What not to do

- **Do not** bulk-convert all stores in one pass
- **Do not** put Tauri event streams in TanStack Query
- **Do not** rewrite shadcn-svelte components
- **Do not** add Zustand, Jotai, or another global state library
- **Do not** migrate to React — see below

---

## Why not React?

| Concern | Svelte (stay) | React (migrate) |
|---------|---------------|-----------------|
| Bundle / startup | Smaller, compile-time reactivity | Heavier typical bundle |
| Existing investment | Svelte 5 runes done, 521 components, shadcn-svelte | Full rewrite of UI layer |
| Tauri integration | Framework-agnostic; invoke layer unchanged | Same invoke, but ~6–12 mo rewrite |
| Hiring / ecosystem | Smaller pool; adequate for desktop app | Larger pool; not worth rewrite cost |
| Complexity source | Domain size + mixed state patterns | Complex apps are hard in any framework |

React migration would rebuild the app without improving performance or addressing root causes (store sprawl, inconsistent data loading).

---

## Per-domain file inventory

Current frontend file counts by domain (approximate):

| Domain | Svelte | TS | Notes |
|--------|--------|-----|-------|
| terminal | 22 | 16 | Phase 6 — last |
| tasks | 21 | 6 | Phase 2 — high store complexity |
| ai | 18 | 10 | After core domains |
| sdk | 16 | 8 | Phase 5 |
| settings | 14 | 6 | Phase 3 |
| cloud | 13 | 20 | Phase 4 |
| projects | 5 | 19 | Phase 1 — pilot |
| dashboard | 0 | 2 | Phase 2 — quick win |
| shared | 3 | 13 | Query infra lives here |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-05 | Initial plan — phased Query migration, store consolidation, domain order |
