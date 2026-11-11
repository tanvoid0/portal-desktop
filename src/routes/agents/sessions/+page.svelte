<!-- Sessions list — persisted agent runs, most recent first. Read-only: no
     launching or mutating here, just browsing history (see `agents/run`). -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { createQuery } from '@tanstack/svelte-query';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { PageEmpty, PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { formatSessionDateTime, formatSessionDateTimeFull } from '$lib/domains/shared/utils';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import History from '~icons/fa6-solid/clock-rotate-left';

  const sessionsQuery = createQuery(agentSessionsQuery);
  const sessions = $derived(sessionsQuery.data ?? []);

  function openSession(id: string) {
    goto(`/agents/sessions/${id}`);
  }

  function onRowKeydown(e: KeyboardEvent, id: string) {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    openSession(id);
  }
</script>

<svelte:head>
  <title>Agent Sessions - Portal Desktop</title>
</svelte:head>

<div class="h-full overflow-auto p-4 md:p-6">
  <div class="space-y-6">
    <PageHeader title="Sessions" description="Past agent runs, most recent first">
      {#snippet actions()}
        <Button
          variant="outline"
          size="sm"
          onclick={() => sessionsQuery.refetch()}
          disabled={sessionsQuery.isFetching}
        >
          <RefreshCw class="mr-2 h-4 w-4 {sessionsQuery.isFetching ? 'animate-spin' : ''}" />
          Refresh
        </Button>
      {/snippet}
    </PageHeader>

    {#if sessionsQuery.isPending}
      <PageLoading message="Loading sessions…" />
    {:else if sessionsQuery.isError}
      <PageError
        message={sessionsQuery.error instanceof Error
          ? sessionsQuery.error.message
          : 'Failed to load sessions'}
        onRetry={() => sessionsQuery.refetch()}
      />
    {:else if sessions.length === 0}
      <PageEmpty
        title="No sessions yet"
        description="Start a run from the Run tab — it will show up here as soon as it starts."
        icon={History}
      />
    {:else}
      <div class="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Backend</TableHead>
              <TableHead>Model</TableHead>
              <TableHead>Cwd</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Started</TableHead>
              <TableHead>Ended</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each sessions as session (session.id)}
              <TableRow
                class="cursor-pointer focus-visible:bg-muted/50 focus-visible:outline-none"
                tabindex={0}
                role="button"
                onclick={() => openSession(session.id)}
                onkeydown={(e) => onRowKeydown(e, session.id)}
              >
                <TableCell class="font-medium">{session.backend_id}</TableCell>
                <TableCell class="text-muted-foreground">{session.model ?? '—'}</TableCell>
                <TableCell class="max-w-xs truncate text-muted-foreground" title={session.cwd}>
                  {session.cwd}
                </TableCell>
                <TableCell>
                  <Badge variant={session.status === 'running' ? 'default' : 'secondary'}>
                    {session.status}
                  </Badge>
                </TableCell>
                <TableCell
                  class="text-muted-foreground"
                  title={formatSessionDateTimeFull(session.started_at)}
                >
                  {formatSessionDateTime(session.started_at)}
                </TableCell>
                <TableCell
                  class="text-muted-foreground"
                  title={session.ended_at ? formatSessionDateTimeFull(session.ended_at) : ''}
                >
                  {session.ended_at ? formatSessionDateTime(session.ended_at) : '—'}
                </TableCell>
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      </div>
    {/if}
  </div>
</div>
