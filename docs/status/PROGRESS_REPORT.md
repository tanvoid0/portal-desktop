# Production Readiness Status

**Last updated:** 2026-06-12  
**Authoritative blocker list:** [IMPROVEMENTS.md](../../IMPROVEMENTS.md)

## Summary

Portal Desktop is **feature-complete for development use** but **not yet production-grade**. Core UX consistency (page shell, toasts, navigation) is done. Remaining work focuses on automated quality gates, backend persistence, and frontend data-layer consolidation.

| Pillar | Status | Notes |
|--------|--------|-------|
| Consistency | In progress | TanStack Query on projects, dashboard, tasks (list); shell on cloud workload lists |
| Stability | In progress | SeaORM Migrator, app-data SQLite path, deployment DB persistence |
| Deprecation-free | In progress | Removed `lucide-svelte`, `@/lib/` in terminal; stub APIs return explicit errors |
| Modularity | In progress | Domain barrel exports, `*Api.ts` layers, network service extraction |

## Completed (this initiative)

- CI workflow: `pnpm check`, `pnpm lint`, `pnpm test:unit`, `cargo test`, `cargo clippy`
- Release workflow: Linux/Windows/macOS builds + GitHub Release publish (tag-triggered)
- Tooling aligned on pnpm; versions synced to `0.1.0`
- SQLite moved to Tauri app data dir with legacy DB migration
- SeaORM `Migrator` + migration smoke test
- Deployment persistence via `deployments` table
- `AppError` adopted in project service (pilot)
- Pipeline execution marks `queued` before background spawn
- TanStack Query provider + projects/dashboard/tasks queries (`createTasksQuery`, `createTaskQuery`)
- Tasks Phase 2 follow-up: `taskUi.svelte.ts` for UI session state; `taskStore.ts` mutations-only
- `ollamaApi.ts` extracted from route page
- `WorkloadListShell` for cloud list pages (pods pilot)
- Frontend unit tests + Playwright smoke test scaffold
- Encryption service round-trip tests

## Phase 2 follow-up (tasks) — done

UI session state extracted to `state/taskUi.svelte.ts` (filters, selection, multi-select, time tracking, templates, saved views, derived helpers). `taskStore.ts` is mutations-only.

## Phase 3 (next — when ready)

Apply the **projects Query template** to form-heavy domains:

| Domain | Approach |
|--------|----------|
| Settings | Single-record read/write via Query + mutation |
| Credentials | Explicit mutations only; no optimistic UI (security-sensitive) |
| Documents | `*Api.ts` + `queries/` + shrink `documentStore.ts` |

Reference: `src/lib/domains/projects/queries/projectQueries.ts`, `state/projectUi.svelte.ts`.

## Open blockers (see IMPROVEMENTS.md)

- Full `AppError` migration across all domains
- Pipeline variables/secrets UI must handle not-implemented API responses
- Remaining cloud workload list pages to adopt `WorkloadListShell`
- Mega-file splits (Terminal.svelte, sdk/+page.svelte, projects/[id])
- Tauri updater still disabled in config
- SDKMAN integration tests require Unix environment

## Do not use this doc for release sign-off

Use the success criteria in the Production Readiness Roadmap plan and green CI on `main` before calling the app production-ready.
