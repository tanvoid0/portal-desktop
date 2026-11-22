<script lang="ts">
  import { createMutation, createQuery, useQueryClient } from '@tanstack/svelte-query';
  import Database from '~icons/fa6-solid/database';
  import Table2 from '~icons/fa6-solid/table';
  import Eye from '~icons/fa6-solid/eye';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import { toast } from '$lib/utils/toast';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import {
    listTables,
    mongoListCollections,
    mongoListDatabases,
    redisListDatabases,
    setEditable,
  } from '../api/databaseApi';
  import { tableList } from '../utils/tableList';
  import type { DbSource } from '../generated';

  interface Props {
    source: DbSource;
    selected?: string | null;
    /** MongoDB only: the database `selected` (a collection) lives in. */
    selectedDb?: string | null;
    onSelect: (sourceId: string, table: string, db?: string) => void;
  }
  const uid = $props.id();
  let { source, selected = null, selectedDb = null, onSelect }: Props = $props();

  const ENGINES: Record<string, { label: string; devicon: string }> = {
    sqlite: { label: 'SQLite', devicon: 'devicon-sqlite-plain' },
    postgres: { label: 'Postgres', devicon: 'devicon-postgresql-plain' },
    mysql: { label: 'MySQL', devicon: 'devicon-mysql-original' },
    mongodb: { label: 'MongoDB', devicon: 'devicon-mongodb-plain' },
    redis: { label: 'Redis', devicon: 'devicon-redis-plain' },
  };
  const engine = $derived(ENGINES[source.engine] ?? { label: source.engine, devicon: '' });

  const isMongo = $derived(source.engine === 'mongodb');
  const isRedis = $derived(source.engine === 'redis');
  const isSql = $derived(!isMongo && !isRedis);
  // Mongo servers can have hundreds of collections: one database open at a time.
  let openDb = $state<string | null>(null);

  const tables = createQuery(() => ({
    queryKey: queryKeys.database.tables(source.id),
    queryFn: () => listTables(source.id),
    enabled: isSql,
  }));
  const redisDbs = createQuery(() => ({
    queryKey: queryKeys.database.redisDatabases(source.id),
    queryFn: () => redisListDatabases(source.id),
    enabled: isRedis,
  }));
  const dbs = createQuery(() => ({
    queryKey: queryKeys.database.mongoDatabases(source.id),
    queryFn: () => mongoListDatabases(source.id),
    enabled: isMongo,
  }));
  const collections = createQuery(() => ({
    queryKey: queryKeys.database.mongoCollections(source.id, openDb ?? ''),
    queryFn: () => mongoListCollections(source.id, openDb!),
    enabled: isMongo && !!openDb,
  }));

  let toggled = $state<boolean>(); // the user's choice; unset follows the size default
  let filter = $state('');
  const allTables = $derived(tables.data ?? []);
  const { big, query, visible, expanded } = $derived(tableList(allTables, filter, toggled));

  function toggle() {
    toggled = !expanded;
    filter = ''; // collapsing a filtered list ends the filter too, or it would just reopen
  }

  const client = useQueryClient();
  const toggleEdit = createMutation(() => ({
    mutationFn: (editable: boolean) => setEditable(source.id, editable),
    onSuccess: () => client.invalidateQueries({ queryKey: queryKeys.database.sources }),
    onError: (e: unknown) => toast.error('Could not toggle editing', String(e)),
  }));
</script>

<div class="space-y-1">
  <div class="flex items-center gap-2 px-2 py-1 text-sm font-medium" title={source.detail}>
    {#if isSql}
      <Button
        variant="ghost"
        size="icon"
        class="size-5"
        aria-label="{source.label} tables"
        aria-expanded={expanded}
        aria-controls="{uid}-tables"
        onclick={toggle}
      >
        <ChevronRight class="h-3 w-3 transition-transform {expanded ? 'rotate-90' : ''}" />
      </Button>
    {/if}
    {#if engine.devicon}
      <TechIcon icon={engine.devicon} size="sm" class="shrink-0" />
    {:else}
      <Database class="h-4 w-4 shrink-0 text-muted-foreground" />
    {/if}
    <span class="min-w-0 flex-1 break-words">{source.label}</span>
    <Badge variant="outline" class="shrink-0 text-[10px] font-normal">{engine.label}</Badge>
  </div>
  <div class="flex items-center gap-2 px-2 text-xs text-muted-foreground">
    <Switch
      id="{uid}-edit"
      checked={source.editable}
      onCheckedChange={(v) => toggleEdit.mutate(v)}
      disabled={toggleEdit.isPending}
      aria-label="Enable editing for {source.label}"
      class="shrink-0"
    />
    <Label for="{uid}-edit" class="text-xs font-normal">
      {source.editable ? 'Editing enabled' : 'Enable editing'}
    </Label>
  </div>
  {#if isRedis}
    {#if redisDbs.isPending}
      <p class="px-6 text-xs text-muted-foreground">Loading...</p>
    {:else if redisDbs.isError}
      <p class="px-6 text-xs text-destructive">{String(redisDbs.error)}</p>
    {:else}
      {#each redisDbs.data ?? [] as d (d.index)}
        <Button
          variant={selected === `db${d.index}` ? 'secondary' : 'ghost'}
          size="sm"
          class="h-7 w-full justify-start gap-2 pl-6"
          onclick={() => onSelect(source.id, `db${d.index}`, String(d.index))}
        >
          <span class="truncate">db{d.index}</span>
          <span class="ml-auto text-xs text-muted-foreground tabular-nums">{d.keys}</span>
        </Button>
      {/each}
    {/if}
  {:else if isMongo}
    {#if dbs.isPending}
      <p class="px-6 text-xs text-muted-foreground">Loading...</p>
    {:else if dbs.isError}
      <p class="px-6 text-xs text-destructive">{String(dbs.error)}</p>
    {:else}
      {#each dbs.data ?? [] as d (d)}
        <Button
          variant="ghost"
          size="sm"
          class="h-7 w-full justify-start gap-2 pl-4"
          aria-expanded={openDb === d}
          onclick={() => (openDb = openDb === d ? null : d)}
        >
          <ChevronRight
            class="h-3 w-3 shrink-0 transition-transform {openDb === d ? 'rotate-90' : ''}"
          />
          <span class="truncate">{d}</span>
        </Button>
        {#if openDb === d}
          {#if collections.isPending}
            <p class="px-9 text-xs text-muted-foreground">Loading...</p>
          {:else if collections.isError}
            <p class="px-9 text-xs text-destructive">{String(collections.error)}</p>
          {:else}
            {#each collections.data ?? [] as c (c)}
              <Button
                variant={selectedDb === d && selected === c ? 'secondary' : 'ghost'}
                size="sm"
                class="h-7 w-full justify-start gap-2 pl-9"
                onclick={() => onSelect(source.id, c, d)}
              >
                <Table2 class="h-3.5 w-3.5 shrink-0" />
                <span class="truncate">{c}</span>
              </Button>
            {/each}
          {/if}
        {/if}
      {/each}
    {/if}
  {:else if tables.isPending}
    <p class="px-6 text-xs text-muted-foreground">Loading...</p>
  {:else if tables.isError}
    <p class="px-6 text-xs text-destructive">{String(tables.error)}</p>
  {:else}
    {#if big}
      <div class="px-2">
        <Input
          bind:value={filter}
          placeholder="Filter {allTables.length} tables..."
          aria-label="Filter {source.label} tables"
          class="h-7 text-xs md:text-xs"
        />
      </div>
      {#if query}
        <p class="px-6 text-xs text-muted-foreground tabular-nums">
          {visible.length ? `${visible.length} of ${allTables.length} tables` : 'No tables match'}
        </p>
      {/if}
    {/if}
    {#if expanded}
      <div id="{uid}-tables" class="space-y-1">
        {#each visible as t (t.name)}
          <Button
            variant={selected === t.name ? 'secondary' : 'ghost'}
            size="sm"
            class="h-7 w-full justify-start gap-2 pl-6"
            onclick={() => onSelect(source.id, t.name)}
          >
            {#if t.kind === 'view'}<Eye class="h-3.5 w-3.5" />{:else}<Table2
                class="h-3.5 w-3.5"
              />{/if}
            <span class="truncate">{t.name}</span>
            <span class="ml-auto text-xs text-muted-foreground">{t.kind}</span>
          </Button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
