<!--
  Apps tab: the modules (apps, services, packages) the scanner found inside a project.
-->

<script lang="ts">
  import { Loader2, RefreshCw, Search } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import type { ProjectModule } from '$lib/domains/projects/types';
  import { stackIcon, stackLabel } from '$lib/domains/projects/utils/moduleStacks';

  interface Props {
    modules: ProjectModule[];
    rescanning: boolean;
    onRescan: () => void;
  }

  let { modules, rescanning, onRescan }: Props = $props();

  const FILTER_THRESHOLD = 12;
  const GROUPS = [
    { role: 'app', title: 'Apps' },
    { role: 'service', title: 'Services' },
    { role: 'package', title: 'Packages' },
  ] as const;

  let filter = $state('');

  const visible = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return modules;
    return modules.filter(
      (m) =>
        m.name.toLowerCase().includes(q) ||
        m.rel_path.toLowerCase().includes(q) ||
        stackLabel(m.stack).toLowerCase().includes(q)
    );
  });
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-center justify-between gap-3">
    {#if modules.length > FILTER_THRESHOLD}
      <div class="relative w-full max-w-xs">
        <Search class="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground" />
        <Input bind:value={filter} placeholder="Filter by name, path or stack" class="pl-8" />
      </div>
    {:else}
      <span></span>
    {/if}
    <Button variant="outline" size="sm" onclick={onRescan} disabled={rescanning} class="gap-2">
      {#if rescanning}
        <Loader2 class="h-4 w-4 animate-spin" />
      {:else}
        <RefreshCw class="h-4 w-4" />
      {/if}
      Rescan
    </Button>
  </div>

  {#if modules.length === 0}
    <div class="rounded-md border border-dashed p-6 text-center text-sm text-muted-foreground">
      No apps found. Portal scans this project folder for Flutter, Android, Node, Rust and other
      apps, services and shared packages, skipping build output and dependency folders.
    </div>
  {:else if visible.length === 0}
    <p class="text-sm text-muted-foreground">No modules match "{filter}".</p>
  {/if}

  {#each GROUPS as group}
    {@const rows = visible.filter((m) => m.role === group.role)}
    {#if rows.length > 0}
      <section class="space-y-2">
        <h3 class="text-sm font-medium text-muted-foreground">
          {group.title} <span class="ml-1 text-xs">{rows.length}</span>
        </h3>
        <ul class="divide-y rounded-md border">
          {#each rows as m (m.id)}
            {@const icon = stackIcon(m.stack)}
            <li class="flex items-center gap-3 px-3 py-2 text-sm">
              <span class="flex h-5 w-5 shrink-0 items-center justify-center">
                {#if icon}<i class="{icon} text-lg"></i>{/if}
              </span>
              <span class="font-medium">{m.name}</span>
              <span class="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground">
                {m.rel_path}
              </span>
              <span class="shrink-0 text-xs text-muted-foreground">{stackLabel(m.stack)}</span>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  {/each}
</div>
