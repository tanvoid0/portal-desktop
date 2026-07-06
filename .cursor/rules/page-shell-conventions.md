# Page Shell Conventions

Use shared shell components from `$lib/components/shell` for list and management pages.

## Required components

| State | Component | Import |
|-------|-----------|--------|
| Page title + actions | `PageHeader` | `$lib/components/shell` |
| Metric row | `PageStats` | `$lib/components/shell` |
| Search + filters | `PageFilters` | `$lib/components/shell` |
| Loading | `PageLoading` | `$lib/components/shell` |
| Error + retry | `PageError` | `$lib/components/shell` |
| Empty + CTA | `PageEmpty` | `$lib/components/shell` |

## Standard page structure

```svelte
<div class="container mx-auto space-y-6 p-6">
  <PageHeader title="..." description="...">
    {#snippet actions()}<!-- primary CTAs -->{/snippet}
  </PageHeader>

  {#if loading}
    <PageLoading message="Loading ..." />
  {:else if error}
    <PageError title="..." message={error} onRetry={load} />
  {:else if items.length === 0}
    <PageEmpty title="..." description="..." actionLabel="..." onAction={...} />
  {:else}
    <!-- content -->
  {/if}
</div>
```

## Feedback and imports

- **Toasts:** `import { toast } from "$lib/utils/toast"` — do not use `$lib/stores/toast` directly
- **Icons:** `@lucide/svelte` — not `lucide-svelte`
- **Path alias:** `$lib/...` — not `@/lib/...` in new or touched files
- **Confirm dialogs:** use `confirmAction()` or `AlertDialog` — not raw `confirm()`

## Navigation

- Main sidebar config lives in `$lib/config/main-nav.ts`
- Automation surfaces: `/automation/blocks`, `/automation/scripts`, `/automation/utilities`
- Legacy routes `/blocks`, `/scripts`, `/utilities` redirect to automation tabs

## Theme

- Theme mode is controlled by `themeStore` (light / dark / system)
- System mode applies `.dark` / `.light` on `<html>` via `resolvedTheme`
- Custom hex colors in ThemeCustomizer apply via `$lib/utils/customTheme.ts` (settings store subscription)

## Out of scope for page shell rules

- Terminal domain (`src/lib/domains/terminal/*`) — separate rework planned
- Cloud domain keeps connection guard and command palette patterns
