<!--
  Step tree of a workflow run, GitHub-Actions style: one column per stage, a
  connector from each step to the steps that need it. Without `executions` it is
  a static preview of the plan; with them each node is a button that selects the
  step's run. Arrow keys move between nodes (roving tabindex).
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { cn } from '$lib/utils';
  import type {
    ScriptExecutionInfo,
    WorkflowPlanStep,
  } from '$lib/domains/scripts/services/scriptExecutionService';
  import {
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from '$lib/domains/projects/pipelines/utils/executionDisplay';
  import { buildStepGraph } from '../stepGraph';

  interface Props {
    plan: WorkflowPlanStep[];
    executions?: ScriptExecutionInfo[];
    selectedStepId?: string | null;
    onSelect?: (stepId: string, executionId?: string) => void;
  }
  let { plan, executions, selectedStepId = null, onSelect = () => {} }: Props = $props();

  const interactive = $derived(executions !== undefined);
  const graph = $derived(buildStepGraph(plan, executions));
  const statusById = $derived(new Map(graph.columns.flat().map((n) => [n.id, n.status])));
  let focusId = $state<string | null>(null);
  const tabStop = $derived.by(() => {
    const ids = new Set(graph.columns.flat().map((n) => n.id));
    return [focusId, selectedStepId].find((id) => id && ids.has(id)) ?? graph.columns[0]?.[0]?.id;
  });

  let box = $state<HTMLDivElement | null>(null);
  let size = $state({ w: 0, h: 0 });
  let paths = $state<{ key: string; d: string; status: string }[]>([]);

  // Connectors are measured from the rendered nodes, so they follow any wrap/resize.
  function measure() {
    if (!box) return;
    const origin = box.getBoundingClientRect();
    const rect = (id: string) =>
      box!.querySelector(`[data-step="${CSS.escape(id)}"]`)?.getBoundingClientRect();
    size = { w: box.scrollWidth, h: box.scrollHeight };
    paths = graph.edges.flatMap(({ from, to }) => {
      const a = rect(from);
      const b = rect(to);
      if (!a || !b) return [];
      const x1 = a.right - origin.left;
      const y1 = a.top + a.height / 2 - origin.top;
      const x2 = b.left - origin.left;
      const y2 = b.top + b.height / 2 - origin.top;
      const mid = (x1 + x2) / 2;
      return [
        {
          key: `${from}>${to}`,
          d: `M${x1} ${y1} C${mid} ${y1} ${mid} ${y2} ${x2} ${y2}`,
          status: statusById.get(from) ?? 'pending',
        },
      ];
    });
  }

  $effect(() => {
    void graph;
    if (!box) return;
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(box);
    return () => ro.disconnect();
  });

  function onKeydown(e: KeyboardEvent) {
    if (!interactive) return;
    const cols = graph.columns;
    const c = cols.findIndex((col) => col.some((n) => n.id === focusId));
    if (c < 0) return;
    const r = cols[c].findIndex((n) => n.id === focusId);
    let next: string | undefined;
    if (e.key === 'ArrowDown') next = cols[c][r + 1]?.id;
    else if (e.key === 'ArrowUp') next = cols[c][r - 1]?.id;
    else if (e.key === 'ArrowRight') next = cols[c + 1]?.[Math.min(r, cols[c + 1].length - 1)]?.id;
    else if (e.key === 'ArrowLeft') next = cols[c - 1]?.[Math.min(r, cols[c - 1].length - 1)]?.id;
    else return;
    e.preventDefault();
    if (!next) return;
    focusId = next;
    box?.querySelector<HTMLElement>(`[data-step="${CSS.escape(next)}"]`)?.focus();
  }
</script>

<div class="overflow-x-auto">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    bind:this={box}
    role="group"
    aria-label="Workflow steps"
    class="relative grid w-max auto-cols-max grid-flow-col gap-x-10 py-1"
    onkeydown={onKeydown}
  >
    <svg
      class="pointer-events-none absolute top-0 left-0"
      width={size.w}
      height={size.h}
      aria-hidden="true"
    >
      {#each paths as p (p.key)}
        <path
          d={p.d}
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          class={cn('transition-colors', getExecutionStatusColor(p.status))}
        />
      {/each}
    </svg>
    {#each graph.columns as column, i (i)}
      <div class="relative flex flex-col justify-center gap-2">
        {#each column as node (node.id)}
          {@const Icon = getExecutionStatusIcon(node.status)}
          <Button
            type="button"
            variant="outline"
            data-step={node.id}
            tabindex={interactive && node.id === tabStop ? 0 : -1}
            aria-current={node.id === selectedStepId ? 'true' : undefined}
            aria-disabled={!interactive}
            class={cn(
              'h-auto min-h-10 justify-start gap-2 bg-background transition-colors',
              !interactive && 'pointer-events-none',
              node.id === selectedStepId && 'ring-2 ring-primary'
            )}
            onfocus={() => (focusId = node.id)}
            onclick={() => onSelect(node.id, node.executionId)}
          >
            <Icon
              class={cn(
                'h-3.5 w-3.5 shrink-0',
                getExecutionStatusColor(node.status),
                node.status === 'running' && 'motion-safe:animate-spin'
              )}
            />
            <span class="truncate">{node.name}</span>
            <span class="text-xs text-muted-foreground">{node.status}</span>
          </Button>
        {/each}
      </div>
    {/each}
  </div>
</div>
