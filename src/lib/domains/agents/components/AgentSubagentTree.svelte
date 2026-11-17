<!-- Subagent tree of one session, built from its persisted events (parent_tool_use_id). -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import Select from '$lib/components/ui/select.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { agentSessionQuery } from '../queries/agentsQueries.js';
  import { buildSubagentTree, type SubagentNode } from '../utils/analytics.js';
  import type { AgentSessionSummary } from '../generated';

  interface Props {
    sessions: AgentSessionSummary[];
  }
  let { sessions }: Props = $props();

  let picked = $state('');
  // Fall back to the newest session when nothing (or a filtered-out session) is picked.
  const id = $derived(sessions.some((s) => s.id === picked) ? picked : (sessions[0]?.id ?? ''));
  const options = $derived(sessions.map((s) => ({ value: s.id, label: s.title || s.id })));
  const detail = createQuery(() => ({ ...agentSessionQuery(id), enabled: !!id }));
  const tree = $derived(buildSubagentTree(detail.data?.events ?? []));
</script>

{#snippet node(n: SubagentNode)}
  <li>
    <div class="flex min-w-0 items-center gap-2 py-0.5 text-sm">
      <span class="truncate font-medium">{n.name}</span>
      <span class="shrink-0 text-xs text-muted-foreground">{n.tools} tool calls</span>
      <Badge variant={n.done ? 'secondary' : 'default'} class="shrink-0">
        {n.done ? 'finished' : 'unfinished'}
      </Badge>
    </div>
    {#if n.children.length}
      <ul class="ml-3 border-l pl-3">
        {#each n.children as c (c.id)}{@render node(c)}{/each}
      </ul>
    {/if}
  </li>
{/snippet}

<section class="min-w-0 space-y-2 rounded-md border bg-card p-3">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <h3 class="text-sm font-medium">Subagents</h3>
    {#if options.length}
      <Select
        bind:value={picked}
        {options}
        placeholder="Latest session"
        aria-label="Session for subagent tree"
        class="w-full sm:w-64"
      />
    {/if}
  </div>
  {#if !id}
    <p class="py-4 text-center text-sm text-muted-foreground">No sessions yet.</p>
  {:else if detail.isPending}
    <p class="py-4 text-center text-sm text-muted-foreground">Loading…</p>
  {:else if detail.isError}
    <p class="py-4 text-center text-sm text-destructive">{String(detail.error)}</p>
  {:else if tree.length === 0}
    <p class="py-4 text-center text-sm text-muted-foreground">This session spawned no subagents.</p>
  {:else}
    <ul>
      {#each tree as n (n.id)}{@render node(n)}{/each}
    </ul>
  {/if}
</section>
