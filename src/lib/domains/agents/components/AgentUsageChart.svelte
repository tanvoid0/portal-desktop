<!-- One metric over time, stacked by model family (dataviz: tokens and cost never share an axis). -->
<script lang="ts">
  import { BarChart } from 'layerchart';
  import { ChartContainer } from '$lib/components/ui/chart';
  import { cn } from '$lib/utils';
  import { CHART_CLASS, FAMILY_CONFIG, compact, money, pivotUsage } from '../utils/analytics.js';
  import type { AgentUsageCell } from '../generated';

  interface Props {
    title: string;
    cells: AgentUsageCell[];
    metric: 'tokens' | 'cost_usd';
  }
  let { title, cells, metric }: Props = $props();

  const fmt = (v: number) => (metric === 'cost_usd' ? money.format(v) : compact.format(v));

  const pivot = $derived(pivotUsage(cells, metric));
  const series = $derived(
    pivot.families.map((key) => ({
      key,
      label: FAMILY_CONFIG[key].label,
      color: `var(--color-${key})`,
    }))
  );
</script>

<section class="min-w-0 space-y-2 rounded-md border bg-card p-3">
  <h3 class="text-sm font-medium">{title}</h3>
  {#if pivot.rows.length === 0}
    <p class="py-8 text-center text-sm text-muted-foreground">No usage recorded yet.</p>
  {:else}
    <ChartContainer
      config={FAMILY_CONFIG}
      class={cn('aspect-auto h-56 w-full', CHART_CLASS)}
      role="img"
      aria-label="{title} per day, stacked by model family"
    >
      <BarChart
        data={pivot.rows}
        x="day"
        {series}
        seriesLayout="stack"
        stackPadding={2}
        legend
        props={{
          xAxis: { format: (d: string) => d.slice(5) },
          bars: { stroke: 'hsl(var(--card))' },
          yAxis: { format: fmt },
        }}
      />
    </ChartContainer>
    <!-- Table view: the light aqua/yellow fall under 3:1 on the surface, so values must be readable without colour. -->
    <table class="sr-only">
      <caption>{title} per day</caption>
      <thead>
        <tr>
          <th>Day</th>
          {#each pivot.families as f (f)}<th>{FAMILY_CONFIG[f].label}</th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each pivot.rows as r (r.day)}
          <tr>
            <td>{r.day}</td>
            {#each pivot.families as f (f)}<td>{fmt(Number(r[f]))}</td>{/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
