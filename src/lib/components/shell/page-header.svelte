<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    title: string;
    description?: string;
    /** Richer alternative to `description` — e.g. a path with a copy button. */
    subtitle?: Snippet;
    badge?: string;
    /** Lets a page keep a status colour on its badge — e.g. `default` while running. */
    badgeVariant?: 'default' | 'secondary' | 'destructive' | 'outline';
    actions?: Snippet;
  }

  let {
    title,
    description,
    subtitle,
    badge,
    badgeVariant = 'secondary',
    actions,
  }: Props = $props();
</script>

<div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
  <div class="flex min-w-0 items-center gap-2">
    <h1 class="min-w-0 truncate text-xl font-semibold">{title}</h1>
    {#if badge}
      <Badge variant={badgeVariant}>{badge}</Badge>
    {/if}
    {#if description}
      <span class="hidden text-muted-foreground/40 sm:inline" aria-hidden="true">|</span>
      <p class="hidden min-w-0 truncate text-sm text-muted-foreground sm:block">{description}</p>
    {/if}
    {#if subtitle}
      <span class="hidden text-muted-foreground/40 sm:inline" aria-hidden="true">|</span>
      <div class="hidden min-w-0 items-center sm:flex">{@render subtitle()}</div>
    {/if}
  </div>
  {#if actions}
    <div class="flex flex-wrap items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
