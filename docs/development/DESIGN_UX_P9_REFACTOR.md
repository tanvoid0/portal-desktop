# P9 Shell Refactor — Completed

**Status:** Done (2026-06-05)  
**Parent:** [`DESIGN_UX_CONSISTENCY.md`](./DESIGN_UX_CONSISTENCY.md)  
**Conventions:** [`.cursor/rules/page-shell-conventions.md`](../../.cursor/rules/page-shell-conventions.md)

Last phase of the design consistency initiative — long-tail detail views, layout-level load/error, and the cloud workloads overview.

---

## Summary

| Phase | Scope | Result |
|-------|--------|--------|
| **9a** | Cloud K8s detail (10 routes) | Page-level `Loading` → `PageLoading` / `PageError` + retry. Tab spinners (YAML, logs, etc.) unchanged. |
| **9b** | Settings layout | `settings/+layout.svelte` — layout-level load/error for all sub-pages. No per-sub-page `PageHeader`. |
| **9c** | Task/document detail | `tasks/[id]`, `tasks/[id]/edit`, `documents/[id]` migrated. |
| **9d** | Form wrappers | Skipped — `TaskForm` / `DocumentEditor` own full-page layout. |
| **9e** | Cloud workloads overview | `cloud/workloads/+page.svelte` — initial load/error shell; refresh keeps dashboard visible, failures toast. |

---

## Files changed

### 9a — Cloud detail (10)

| Route | File |
|-------|------|
| Secret | `src/routes/cloud/secrets/[secret]/+page.svelte` |
| ConfigMap | `src/routes/cloud/configmaps/[configmap]/+page.svelte` |
| Ingress | `src/routes/cloud/ingress/[ingress]/+page.svelte` |
| Pod | `src/routes/cloud/workloads/pods/[pod]/+page.svelte` |
| Deployment | `src/routes/cloud/workloads/deployments/[deployment]/+page.svelte` |
| Service | `src/routes/cloud/workloads/services/[service]/+page.svelte` |
| Job | `src/routes/cloud/workloads/jobs/[job]/+page.svelte` |
| CronJob | `src/routes/cloud/workloads/cronjobs/[cronjob]/+page.svelte` |
| StatefulSet | `src/routes/cloud/workloads/statefulsets/[statefulset]/+page.svelte` |
| DaemonSet | `src/routes/cloud/workloads/daemonsets/[daemonset]/+page.svelte` |

### 9b — Settings

- `src/routes/settings/+layout.svelte`
- `src/routes/settings/theme/+page.svelte` — removed redundant sub-page `PageLoading` (layout owns load/error)

### 9c — Tasks & documents

- `src/routes/tasks/[id]/+page.svelte`
- `src/routes/tasks/[id]/edit/+page.svelte`
- `src/routes/documents/[id]/+page.svelte`

### 9e — Cloud overview

- `src/routes/cloud/workloads/+page.svelte`

---

## Pattern applied

**Detail pages (9a, 9c):**

```svelte
import { PageLoading, PageError } from "$lib/components/shell";
import Loading from "$lib/components/ui/loading.svelte"; // keep for tab-level only

{#if isLoading}
  <PageLoading message="Loading secret..." />
{:else if error}
  <PageError title="Failed to load secret" message={error} onRetry={loadSecret} />
{:else}
  <!-- content; tab actions may still use <Loading /> -->
{/if}
```

**Layout-level (9b):**

```svelte
{#if $isLoadingSettings}
  <PageLoading message="Loading settings..." />
{:else if $settingsError}
  <PageError title="Failed to load settings" message={$settingsError} onRetry={() => settingsActions.loadSettings()} />
{:else}
  {@render children()}
{/if}
```

**Overview with refresh (9e):**

```svelte
{#if !connected}
  <!-- disconnected empty state -->
{:else if loadError && !hasLoadedOnce}
  <PageError title="Failed to load cluster data" message={loadError} onRetry={loadClusterData} />
{:else if isLoadingData && !hasLoadedOnce}
  <PageLoading message="Loading cluster data..." />
{:else}
  <!-- dashboard; refresh failures use toast, not full-page error -->
{/if}
```

Load is triggered by a `wasConnected` guard in `$effect` — fires when the cluster first connects or reconnects, not on every store refresh (avoids reload loops).

---

## Intentionally unchanged

- `CloudConnectionGuard`, command palette, `BaseResourceTable`
- Cloud tab-level spinners (YAML editor load, pod logs, metrics panels)
- Settings sub-page `h2` section titles (layout owns top bar); sub-pages must not duplicate layout-level `PageLoading` / `PageError`
- `TaskForm` / `DocumentEditor` full-page layouts (9d)
- AI wizards: `TaskGenerationPage`, `DocumentGenerationPage` — step-level `LoadingSpinner` is appropriate
- Terminal domain — separate epic

---

## Verification

- [x] **9a** — 10 cloud detail routes: page-level `PageLoading` / `PageError`; tab-level `Loading` only
- [x] **9b** — `settings/+layout.svelte` owns load/error; sub-pages (e.g. theme) do not duplicate shell
- [x] **9c** — `tasks/[id]`, `tasks/[id]/edit`, `documents/[id]` use shell + retry
- [x] **9d** — Form wrappers reviewed and skipped
- [x] **9e** — `cloud/workloads/+page.svelte` shell + toast-on-refresh + `wasConnected` guard

---

## Smoke test (P9-specific)

With `pnpm tauri dev` and a connected cluster:

1. **Cloud detail** — open any K8s resource → `PageLoading`, then content; disconnect → `PageError` + retry works.
2. **Cloud overview** — `/cloud/workloads` → `PageLoading` on first load; Refresh keeps dashboard visible.
3. **Settings** — simulate load failure → `PageError` in content area; sub-pages inherit layout shell.
4. **Tasks** — `/tasks/[id]` and `/tasks/[id]/edit` → load/error/retry consistent with project detail.
5. **Documents** — `/documents/[id]` → same pattern.

---

## Deferred (post-P9)

See parent doc [Follow-up table](./DESIGN_UX_CONSISTENCY.md#follow-up-optional):

| Item | Notes |
|------|-------|
| Terminal domain | Separate epic — imports, confirm API, icons |
| `sdk/+page.svelte`, `sdk/ai/ollama` | Large specialty pages; migrate when touched |
| `tasks/[id]/generate`, `TaskGenerationPage` | Wizard step spinners — not page-level shell |
| `documents/generate` | Thin wrapper around `DocumentGenerationPage` — no page-level load needed |
| Cloud emoji nav → Lucide | Cosmetic only — skip |
| TanStack Query migration | Architecture — [`SVELTE_MAINTAINABILITY_PLAN.md`](./SVELTE_MAINTAINABILITY_PLAN.md) |
