<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import Database from '~icons/fa6-solid/database';
  import Table2 from '~icons/fa6-solid/table';
  import Eye from '~icons/fa6-solid/eye';
  import { Button } from '$lib/components/ui/button';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { listTables } from '../api/databaseApi';
  import type { DbSource } from '../generated';

  interface Props {
    source: DbSource;
    selected?: string | null;
    onSelect: (sourceId: string, table: string) => void;
  }
  let { source, selected = null, onSelect }: Props = $props();

  const tables = createQuery(() => ({
    queryKey: queryKeys.database.tables(source.id),
    queryFn: () => listTables(source.id),
  }));
</script>

<div class="space-y-1">
  <div class="flex items-center gap-2 px-2 py-1 text-sm font-medium" title={source.detail}>
    <Database class="h-4 w-4 shrink-0" />
    <span class="truncate">{source.label}</span>
  </div>
  {#if tables.isPending}
    <p class="px-6 text-xs text-muted-foreground">Loading...</p>
  {:else if tables.isError}
    <p class="px-6 text-xs text-destructive">{String(tables.error)}</p>
  {:else}
    {#each tables.data ?? [] as t (t.name)}
      <Button
        variant={selected === t.name ? 'secondary' : 'ghost'}
        size="sm"
        class="h-7 w-full justify-start gap-2 pl-6"
        onclick={() => onSelect(source.id, t.name)}
      >
        {#if t.kind === 'view'}<Eye class="h-3.5 w-3.5" />{:else}<Table2 class="h-3.5 w-3.5" />{/if}
        <span class="truncate">{t.name}</span>
        <span class="ml-auto text-xs text-muted-foreground">{t.kind}</span>
      </Button>
    {/each}
  {/if}
</div>
