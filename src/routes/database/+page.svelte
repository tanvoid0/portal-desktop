<script lang="ts">
  import { createMutation, createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { open } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Tabs from '$lib/components/ui/tabs';
  import { PageHeader, PageLoading, PageError, PageEmpty } from '$lib/components/shell';
  import { toast } from '$lib/utils/toast';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import {
    describeTable,
    fetchRows,
    listSources,
    openSqlite,
    runQuery,
  } from '$lib/domains/database/api/databaseApi';
  import ResultGrid from '$lib/domains/database/components/ResultGrid.svelte';
  import SourceNode from '$lib/domains/database/components/SourceNode.svelte';

  const PAGE_SIZE = 100;
  const client = useQueryClient();

  let sel = $state<{ sourceId: string; table: string } | null>(null);
  let page = $state(0);
  let sql = $state('');
  let tab = $state('data');

  const sources = createQuery(() => ({
    queryKey: queryKeys.database.sources,
    queryFn: listSources,
  }));
  const columns = createQuery(() => ({
    queryKey: queryKeys.database.columns(sel?.sourceId ?? '', sel?.table ?? ''),
    queryFn: () => describeTable(sel!.sourceId, sel!.table),
    enabled: !!sel,
  }));
  const rows = createQuery(() => ({
    queryKey: queryKeys.database.rows(sel?.sourceId ?? '', sel?.table ?? '', page),
    queryFn: () => fetchRows(sel!.sourceId, sel!.table, PAGE_SIZE, page * PAGE_SIZE),
    enabled: !!sel,
  }));

  // Query tab runs against the selected source, else the first one.
  const querySourceId = $derived(sel?.sourceId ?? sources.data?.[0]?.id ?? '');
  const run = createMutation(() => ({
    mutationFn: () => runQuery(querySourceId, sql),
    onError: (e: unknown) => toast.error('Query failed', String(e)),
  }));
  const openDb = createMutation(() => ({
    mutationFn: openSqlite,
    onSuccess: () => client.invalidateQueries({ queryKey: queryKeys.database.sources }),
    onError: (e: unknown) => toast.error('Could not open database', String(e)),
  }));

  function select(sourceId: string, table: string) {
    sel = { sourceId, table };
    page = 0;
    tab = 'data';
  }

  async function pickFile() {
    const path = await open({
      multiple: false,
      filters: [{ name: 'SQLite', extensions: ['db', 'sqlite', 'sqlite3'] }],
    });
    if (typeof path === 'string') openDb.mutate(path);
  }
</script>

<svelte:head>
  <title>Database - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto space-y-6 p-6">
  <PageHeader title="Database Explorer" description="Browse SQLite databases (read-only)">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={pickFile} disabled={openDb.isPending}>
        Open .db file…
      </Button>
    {/snippet}
  </PageHeader>

  {#if sources.isPending}
    <PageLoading message="Loading sources..." />
  {:else if sources.isError}
    <PageError
      title="Failed to load sources"
      message={String(sources.error)}
      onRetry={() => sources.refetch()}
    />
  {:else if sources.data.length === 0}
    <PageEmpty
      title="No databases"
      description="Open a SQLite file to start."
      actionLabel="Open .db file…"
      onAction={pickFile}
    />
  {:else}
    <div class="grid grid-cols-[16rem_1fr] gap-4">
      <aside class="space-y-3 rounded-md border p-2">
        {#each sources.data as source (source.id)}
          <SourceNode
            {source}
            selected={sel?.sourceId === source.id ? sel.table : null}
            onSelect={select}
          />
        {/each}
      </aside>

      <section class="min-w-0 space-y-3">
        <Tabs.Root bind:value={tab}>
          <Tabs.List>
            <Tabs.Trigger value="data">Data</Tabs.Trigger>
            <Tabs.Trigger value="query">Query</Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="data" class="space-y-3">
            {#if !sel}
              <p class="text-sm text-muted-foreground">Select a table or view.</p>
            {:else if rows.isPending}
              <PageLoading message="Loading rows..." />
            {:else if rows.isError}
              <PageError message={String(rows.error)} onRetry={() => rows.refetch()} />
            {:else}
              <ResultGrid result={rows.data} columnInfo={columns.data} />
              <div class="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page === 0}
                  onclick={() => (page -= 1)}>Prev</Button
                >
                <span class="text-xs text-muted-foreground">Page {page + 1}</span>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={rows.data.rows.length < PAGE_SIZE}
                  onclick={() => (page += 1)}>Next</Button
                >
              </div>
            {/if}
          </Tabs.Content>

          <Tabs.Content value="query" class="space-y-3">
            <Textarea
              bind:value={sql}
              rows={6}
              class="font-mono text-sm"
              placeholder="SELECT * FROM ..."
            />
            <Button
              size="sm"
              onclick={() => run.mutate()}
              disabled={run.isPending || !sql.trim() || !querySourceId}
            >
              Run
            </Button>
            {#if run.data}
              <ResultGrid result={run.data} />
            {/if}
          </Tabs.Content>
        </Tabs.Root>
      </section>
    </div>
  {/if}
</div>
