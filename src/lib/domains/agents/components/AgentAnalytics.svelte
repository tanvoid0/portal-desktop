<!-- Analytics (#192): usage, tool calls and outcomes aggregated in SQL over agent_events,
     honoring the overview's backend filter. Deferred: Sankey, agent_workflows ingest (4b). -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { BarChart } from 'layerchart';
  import { PageLoading, PageError } from '$lib/components/shell';
  import { Badge } from '$lib/components/ui/badge';
  import { ChartContainer } from '$lib/components/ui/chart';
  import { agentAnalyticsQuery } from '../queries/agentsQueries.js';
  import { FAMILY_CONFIG, familyOf } from '../utils/analytics.js';
  import AgentUsageChart from './AgentUsageChart.svelte';
  import AgentSubagentTree from './AgentSubagentTree.svelte';
  import type { AgentSessionSummary } from '../generated';

  interface Props {
    /** '' = all backends. */
    backendId?: string;
    /** Sessions in scope, for the subagent tree. */
    sessions: AgentSessionSummary[];
  }
  let { backendId = '', sessions }: Props = $props();

  const q = createQuery(() => agentAnalyticsQuery(backendId));

  const compact = new Intl.NumberFormat(undefined, { notation: 'compact' });
  const money = new Intl.NumberFormat(undefined, { style: 'currency', currency: 'USD' });
  const bad = (s: string) => s === 'failed' || s === 'interrupted';
  const duration = (secs: number) =>
    secs < 90
      ? `${Math.round(secs)}s`
      : secs < 5400
        ? `${Math.round(secs / 60)}m`
        : `${(secs / 3600).toFixed(1)}h`;

  // Per model id (not family) so the table shows what actually ran; swatch = family colour.
  const byModel = $derived.by(() => {
    const m = new Map<string, { tokens: number; cache: number; cost: number }>();
    for (const c of q.data?.usage ?? []) {
      const r = m.get(c.model) ?? { tokens: 0, cache: 0, cost: 0 };
      r.tokens += c.tokens;
      r.cache += c.cache_tokens;
      r.cost += c.cost_usd;
      m.set(c.model, r);
    }
    return [...m].sort((a, b) => b[1].cost - a[1].cost);
  });
  const toolConfig = { count: { label: 'Calls', color: 'hsl(var(--muted-foreground))' } };
</script>

<section class="space-y-3">
  <h2 class="text-sm font-medium text-muted-foreground">Analytics</h2>
  {#if q.isPending}
    <PageLoading message="Loading analytics..." />
  {:else if q.isError}
    <PageError message={String(q.error)} onRetry={() => q.refetch()} />
  {:else}
    {@const d = q.data}
    <div class="flex flex-wrap items-center gap-2 text-sm">
      {#each d.outcomes as o (o.status)}
        <Badge variant={bad(o.status) ? 'destructive' : 'secondary'}>{o.status} {o.count}</Badge>
      {/each}
      {#if d.avg_duration_secs != null}
        <span class="text-xs text-muted-foreground">avg run {duration(d.avg_duration_secs)}</span>
      {/if}
    </div>

    <div class="grid gap-3 lg:grid-cols-2">
      <AgentUsageChart title="Tokens" cells={d.usage} metric="tokens" />
      <AgentUsageChart title="Cost" cells={d.usage} metric="cost_usd" />
    </div>

    <div class="grid gap-3 lg:grid-cols-2">
      <section class="min-w-0 space-y-2 rounded-md border bg-card p-3">
        <h3 class="text-sm font-medium">Usage by model</h3>
        {#if byModel.length === 0}
          <p class="py-4 text-center text-sm text-muted-foreground">No usage recorded yet.</p>
        {:else}
          <!-- ChartContainer only to scope the family colour vars to the swatches. -->
          <ChartContainer config={FAMILY_CONFIG} class="block aspect-auto overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="text-left text-xs text-muted-foreground">
                <tr>
                  <th class="py-1 font-normal">Model</th>
                  <th class="py-1 text-right font-normal">Tokens</th>
                  <th class="py-1 text-right font-normal">Cache</th>
                  <th class="py-1 text-right font-normal">Cost</th>
                </tr>
              </thead>
              <tbody class="tabular-nums">
                {#each byModel as [model, r] (model)}
                  <tr class="border-t">
                    <td class="py-1">
                      <span class="flex min-w-0 items-center gap-2">
                        <span
                          class="size-2.5 shrink-0 rounded-[2px]"
                          style:background-color="var(--color-{familyOf(model)})"
                        ></span>
                        <span class="truncate">{model}</span>
                      </span>
                    </td>
                    <td class="py-1 text-right">{compact.format(r.tokens)}</td>
                    <td class="py-1 text-right">{compact.format(r.cache)}</td>
                    <td class="py-1 text-right">{money.format(r.cost)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </ChartContainer>
        {/if}
      </section>

      <section class="min-w-0 space-y-2 rounded-md border bg-card p-3">
        <h3 class="text-sm font-medium">Tool calls</h3>
        {#if d.tools.length === 0}
          <p class="py-4 text-center text-sm text-muted-foreground">No tool calls yet.</p>
        {:else}
          <ChartContainer
            config={toolConfig}
            class="aspect-auto w-full"
            style="height: {Math.max(8, d.tools.length * 1.75)}rem"
            role="img"
            aria-label="Tool call counts, most used first"
          >
            <BarChart
              data={d.tools}
              orientation="horizontal"
              y="name"
              series={[{ key: 'count', label: 'Calls', color: 'var(--color-count)' }]}
              props={{ yAxis: { format: (n: string) => n.slice(0, 18) } }}
              labels
            />
          </ChartContainer>
        {/if}
      </section>
    </div>

    <AgentSubagentTree {sessions} />
  {/if}
</section>
