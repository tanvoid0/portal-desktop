<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import Database from '~icons/fa6-solid/database';
  import Server from '~icons/fa6-solid/server';
  import Fish from '~icons/fa6-solid/fish';
  import Table2 from '~icons/fa6-solid/table';
  import Eye from '~icons/fa6-solid/eye';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { listTables } from '../api/databaseApi';
  import type { DbSource } from '../generated';

  interface Props {
    source: DbSource;
    selected?: string | null;
    onSelect: (sourceId: string, table: string) => void;
  }
  let { source, selected = null, onSelect }: Props = $props();

  // Engine presentation. No fa6 mascots for pg/mysql, so pick available icons
  // and let the badge carry the engine name.
  const ENGINES: Record<string, { label: string; icon: typeof Database; color: string }> = {
    sqlite: { label: 'SQLite', icon: Database, color: 'text-muted-foreground' },
    postgres: { label: 'Postgres', icon: Server, color: 'text-sky-500' },
    mysql: { label: 'MySQL', icon: Fish, color: 'text-amber-500' },
  };
  const engine = $derived(
    ENGINES[source.engine] ?? {
      label: source.engine,
      icon: Database,
      color: 'text-muted-foreground',
    }
  );
  const Icon = $derived(engine.icon);

  const tables = createQuery(() => ({
    queryKey: queryKeys.database.tables(source.id),
    queryFn: () => listTables(source.id),
  }));
</script>

<div class="space-y-1">
  <div class="flex items-center gap-2 px-2 py-1 text-sm font-medium" title={source.detail}>
    <Icon class={`h-4 w-4 shrink-0 ${engine.color}`} />
    <span class="truncate">{source.label}</span>
    <Badge variant="outline" class="ml-auto shrink-0 text-[10px] font-normal">{engine.label}</Badge>
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
