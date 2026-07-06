# Design & UX Consistency — Handoff

**Status:** Complete for in-scope surfaces (P0–P9, 2026-06-05)  
**Scope:** App-wide visual/UX consistency — **terminal domain excluded**  
**Conventions:** [`.cursor/rules/page-shell-conventions.md`](../../.cursor/rules/page-shell-conventions.md)

Handoff doc for continuing optional polish or separate epics. Original plan: `.cursor/plans/design_ux_consistency_24642a79.plan.md` (do not edit).

---

## Summary

Unified Portal Desktop around one page shell, one toast API, one theme system, and clearer navigation (automation hub, AI history, sidebar labels).

| Area | Done |
|------|------|
| **Foundation** | Shell components (`PageHeader`, `PageLoading`, `PageError`, `PageEmpty`, `NavSectionList`); `$lib/utils/toast`; `@lucide/svelte` (non-terminal); theme tokens + custom hex via `customTheme.ts`; cursor rule |
| **List & hub pages** | Home, projects, tasks, documents, credentials, automation (blocks/scripts/utilities), SDK manager/versions, deployments lists, AI hub |
| **Detail & forms** | Project detail/create/edit; SDK sub-routes + manager detail; cloud detail (10 K8s routes); settings layout; task/document detail |
| **Cloud lists** | `BaseResourceTable` empty/filtered states; `confirmAction()` on destructive actions |
| **Hygiene** | `@/lib/` → `$lib/` (routes + domains, except terminal); `svelte-sonner` removed; raw `confirm()` → `confirmAction()` (non-terminal) |
| **P9 long-tail** | See [P9 record](./DESIGN_UX_P9_REFACTOR.md) — cloud detail, workloads overview, settings layout, task/doc detail |

---

## Shell coverage (quick map)

**Full shell** (header + load/error/empty where applicable): projects, tasks, credentials, documents list, automation tabs, SDK manager/versions, most SDK `[sdk]/*` sub-routes, AI hub, create forms (`deployments/new`, `cloud/secrets/new`, `cloud/configmaps/new`).

**Load/error only** (custom action headers kept): project detail, cloud K8s detail routes, cloud workloads overview, task/document detail, settings (layout-level).

**Intentionally unchanged:** settings sub-page `h2` section titles; cloud tab-level spinners (YAML, logs); `CloudConnectionGuard`; terminal domain.

---

## Follow-up (optional)

Migrate opportunistically when touching a file — no blocking work remains for the consistency initiative.

| Item | Priority | Notes |
|------|----------|-------|
| **Terminal domain rework** | Separate epic | Imports, confirm API, icons, UX — not shell polish |
| **`sdk/+page.svelte`, `sdk/ai/ollama`** | Low | Large specialty pages |
| **`tasks/[id]/generate`, `TaskGenerationPage`** | Low | Wizard; inline `LoadingSpinner` for step load |
| **`documents/generate`** | Low | Check if page-level load/error needed |
| **Cloud emoji nav → Lucide** | Skip | Cosmetic only |
| **TanStack Query migration** | Architecture | [`SVELTE_MAINTAINABILITY_PLAN.md`](./SVELTE_MAINTAINABILITY_PLAN.md) |
| **Automation folder naming** | Nice-to-have | Workflow automation vs hub pages share `domains/automation/` |
| **shadcn component consistency** | Complete | [`SHADCN_COMPONENT_AUDIT.md`](./SHADCN_COMPONENT_AUDIT.md); [shadcn cursor rule](../../.cursor/rules/shadcn-component-conventions.md) |

---

## Key files

| Purpose | Path |
|---------|------|
| Page shell | `src/lib/components/shell/` |
| Conventions rule | `.cursor/rules/page-shell-conventions.md` |
| Main / AI nav | `src/lib/config/main-nav.ts`, `ai-nav.ts` |
| Root layout | `src/routes/+layout.svelte` |
| Toast API | `src/lib/utils/toast.ts` |
| Theme | `themeStore.ts`, `customTheme.ts`, `app.css` |
| Cloud table empty state | `BaseResourceTable.svelte` |
| Settings shell | `src/routes/settings/+layout.svelte` |

---

## Standard page template

```svelte
<script lang="ts">
  import { PageHeader, PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import { toast } from "$lib/utils/toast";

  let loading = $state(false);
  let error = $state<string | null>(null);

  async function load() {
    loading = true;
    error = null;
    try {
      /* fetch */
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load";
      toast.error("Failed to load", error);
    } finally {
      loading = false;
    }
  }
</script>

<div class="container mx-auto space-y-6 p-6">
  <PageHeader title="My Page" description="..." />

  {#if loading}
    <PageLoading message="Loading..." />
  {:else if error}
    <PageError title="Failed to load" message={error} onRetry={load} />
  {:else if items.length === 0}
    <PageEmpty title="No items" description="..." actionLabel="Create" onAction={create} />
  {:else}
    <!-- content -->
  {/if}
</div>
```

---

## Smoke test

With `pnpm tauri dev`:

- Theme: Settings → System follows OS dark/light
- Nav: Automation tabs; `/blocks` → `/automation/blocks`; AI History works
- Shell: Projects, Tasks, Credentials, Documents — consistent loading/error/empty
- Cloud detail: disconnected cluster → `PageError` with retry
- Cloud overview: `/cloud/workloads` → `PageLoading` on first load, refresh keeps dashboard
- Settings: load failure → retry in layout content area

---

## Thread prompt (follow-up work)

> Optional shell polish from `docs/development/DESIGN_UX_CONSISTENCY.md` follow-up table. Follow `.cursor/rules/page-shell-conventions.md`. Terminal domain out of scope unless explicitly requested.

---

## Related docs

- [P9 completion record](./DESIGN_UX_P9_REFACTOR.md)
- [Svelte Maintainability Plan](./SVELTE_MAINTAINABILITY_PLAN.md)
- [Svelte Patterns Review](./SVELTE_PATTERNS_REVIEW.md)
