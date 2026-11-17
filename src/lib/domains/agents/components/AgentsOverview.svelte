<!-- Cross-backend agents overview (#871): the mount point for monitoring views.
     Backend is a filter here, not one copy per engine tab. -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { goto } from '$app/navigation';
  import { agentTabPath } from '$lib/config/agents-tabs';
  import {
    PageContainer,
    PageHeader,
    PageStats,
    PageLoading,
    PageError,
    PageEmpty,
  } from '$lib/components/shell';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import Select from '$lib/components/ui/select.svelte';
  import { formatSessionDateTime } from '$lib/domains/shared/utils';
  import { agentBackendsQuery, agentSessionsQuery } from '../queries/agentsQueries.js';
  import AgentsBoard from './AgentsBoard.svelte';
  import AgentActivityFeed from './AgentActivityFeed.svelte';
  import { overviewSessions } from '../utils/overviewStats.js';
  import { basename } from '../utils/groupSessions.js';
  import type { AgentSessionSummary } from '../generated';

  const sessionsQuery = createQuery(agentSessionsQuery);
  const backendsQuery = createQuery(agentBackendsQuery);

  // The Select wrapper shows its placeholder for a falsy value, so "all" can't be ''.
  const ALL = 'all';
  let selected = $state(ALL);
  const backend = $derived(selected === ALL ? '' : selected);
  const options = $derived([
    { value: ALL, label: 'All backends' },
    ...(backendsQuery.data ?? []).map((b) => ({ value: b.id, label: b.display_name })),
  ]);
  const scoped = $derived(
    (sessionsQuery.data ?? []).filter((s) => !backend || s.backend_id === backend)
  );
  const view = $derived(overviewSessions(sessionsQuery.data ?? [], backend));
  const stats = $derived([
    { label: 'Total', value: view.stats.total },
    { label: 'Running', value: view.stats.running },
    { label: 'Interrupted', value: view.stats.interrupted },
    { label: 'Failed', value: view.stats.failed },
  ]);

  const backendName = (id: string) =>
    backendsQuery.data?.find((b) => b.id === id)?.display_name ?? id;
  const variant = (s: AgentSessionSummary) =>
    s.status === 'running'
      ? 'default'
      : s.status === 'interrupted' || s.exit_code
        ? 'destructive'
        : 'secondary';
  const open = (s: AgentSessionSummary) =>
    goto(agentTabPath(s.backend_id) + '?session=' + encodeURIComponent(s.id));
</script>

{#snippet list(title: string, rows: AgentSessionSummary[])}
  {#if rows.length}
    <section class="space-y-2">
      <h2 class="text-sm font-medium text-muted-foreground">{title}</h2>
      <ul class="divide-y rounded-md border bg-card">
        {#each rows as s (s.id)}
          <li>
            <Button
              variant="ghost"
              class="h-auto w-full min-w-0 justify-between gap-3 rounded-none px-3 py-2 text-left font-normal"
              onclick={() => open(s)}
            >
              <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-medium">{s.title || s.backend_id}</span>
                <span class="block truncate text-xs text-muted-foreground">
                  {backend ? '' : `${backendName(s.backend_id)} · `}{basename(s.cwd)} · {formatSessionDateTime(
                    s.started_at
                  )}
                </span>
              </span>
              <Badge variant={variant(s)} class="shrink-0">{s.status}</Badge>
            </Button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
{/snippet}

<PageContainer class="space-y-4 py-4">
  <PageHeader title="Agents overview" description="Every agent run, across backends.">
    {#snippet actions()}
      <Select
        bind:value={selected}
        {options}
        aria-label="Filter by backend"
        class="w-full sm:w-48"
      />
    {/snippet}
  </PageHeader>

  {#if sessionsQuery.isPending}
    <PageLoading message="Loading sessions..." />
  {:else if sessionsQuery.isError}
    <PageError message={String(sessionsQuery.error)} onRetry={() => sessionsQuery.refetch()} />
  {:else if view.stats.total === 0}
    <PageEmpty
      title={backend ? `No ${backendName(backend)} runs yet` : 'No agent runs yet'}
      description="Runs from any backend show up here."
    />
  {:else}
    <PageStats {stats} />
    {@render list('Running', view.running)}
    {@render list('Recent', view.recent)}
    <AgentsBoard sessions={[...view.running, ...view.recent]} {backendName} onOpen={open} />
    <AgentActivityFeed sessions={scoped} backendId={backend} onOpen={open} />
  {/if}
</PageContainer>
