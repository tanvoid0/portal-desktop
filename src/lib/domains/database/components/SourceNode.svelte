<script lang="ts">
  import { createMutation, createQuery, useQueryClient } from '@tanstack/svelte-query';
  import Database from '~icons/fa6-solid/database';
  import Table2 from '~icons/fa6-solid/table';
  import Eye from '~icons/fa6-solid/eye';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Switch } from '$lib/components/ui/switch';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import { toast } from '$lib/utils/toast';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { listTables, setEditable } from '../api/databaseApi';
  import type { DbSource } from '../generated';

  interface Props {
    source: DbSource;
    selected?: string | null;
    onSelect: (sourceId: string, table: string) => void;
  }
  let { source, selected = null, onSelect }: Props = $props();

  const ENGINES: Record<string, { label: string; devicon: string }> = {
    sqlite: { label: 'SQLite', devicon: 'devicon-sqlite-plain' },
    postgres: { label: 'Postgres', devicon: 'devicon-postgresql-plain' },
    mysql: { label: 'MySQL', devicon: 'devicon-mysql-original' },
  };
  const engine = $derived(ENGINES[source.engine] ?? { label: source.engine, devicon: '' });

  const tables = createQuery(() => ({
    queryKey: queryKeys.database.tables(source.id),
    queryFn: () => listTables(source.id),
  }));

  const client = useQueryClient();
  const toggleEdit = createMutation(() => ({
    mutationFn: (editable: boolean) => setEditable(source.id, editable),
    onSuccess: () => client.invalidateQueries({ queryKey: queryKeys.database.sources }),
    onError: (e: unknown) => toast.error('Could not toggle editing', String(e)),
  }));
</script>

<div class="space-y-1">
  <div class="flex items-center gap-2 px-2 py-1 text-sm font-medium" title={source.detail}>
    {#if engine.devicon}
      <TechIcon icon={engine.devicon} size="sm" class="shrink-0" />
    {:else}
      <Database class="h-4 w-4 shrink-0 text-muted-foreground" />
    {/if}
    <span class="flex-1 truncate">{source.label}</span>
    <Switch
      checked={source.editable}
      onCheckedChange={(v) => toggleEdit.mutate(v)}
      disabled={toggleEdit.isPending}
      title={source.editable ? 'Editing enabled' : 'Enable editing'}
      class="shrink-0"
    />
    <Badge variant="outline" class="shrink-0 text-[10px] font-normal">{engine.label}</Badge>
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
