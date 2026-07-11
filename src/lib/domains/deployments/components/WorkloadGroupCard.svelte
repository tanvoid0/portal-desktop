<script lang="ts">
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "$lib/components/ui/collapsible";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
  } from "$lib/components/ui/card";
  import type { WorkloadGroup } from "../utils/workloadGrouping";
  import {
    computeGroupRollup,
    kindBadgeLabel,
  } from "../utils/workloadGrouping";
  import { fmtBytes, fmtPercent, isContainerRunning } from "../utils/format";
  import ContainerRow from "./ContainerRow.svelte";
  import {
    ChevronDown,
    Play,
    Square,
    FolderOpen,
    Layers,
    Cpu,
    MemoryStick,
  } from "@lucide/svelte";

  interface Props {
    group: WorkloadGroup;
    defaultOpen?: boolean;
    onStart?: (id: string) => void;
    onStop?: (id: string) => void;
    onRemove?: (id: string) => void;
    onStartDeployment?: (id: string) => void;
    onStopDeployment?: (id: string) => void;
  }

  let {
    group,
    defaultOpen = true,
    onStart,
    onStop,
    onRemove,
    onStartDeployment,
    onStopDeployment,
  }: Props = $props();

  let open = $state(defaultOpen);

  const rollup = $derived(computeGroupRollup(group.containers));
  const hasRunning = $derived(rollup.running > 0);
  const hasStopped = $derived(rollup.running < rollup.total);

  async function startAll() {
    for (const container of group.containers) {
      if (!isContainerRunning(container.status)) {
        await onStart?.(container.id);
      }
    }
  }

  async function stopAll() {
    for (const container of group.containers) {
      if (isContainerRunning(container.status)) {
        await onStop?.(container.id);
      }
    }
  }
</script>

<Collapsible bind:open>
  <Card class="overflow-hidden transition-shadow hover:shadow-md">
    <CollapsibleTrigger class="w-full text-left">
      <CardHeader class="cursor-pointer space-y-3 pb-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-2">
              {#if group.kind === "portal"}
                <FolderOpen class="h-4 w-4 shrink-0 text-muted-foreground" />
              {:else if group.kind === "compose"}
                <Layers class="h-4 w-4 shrink-0 text-muted-foreground" />
              {/if}
              <h3 class="truncate text-lg font-semibold">{group.name}</h3>
              <Badge variant="outline">{kindBadgeLabel(group.kind)}</Badge>
              <Badge
                variant={rollup.running === rollup.total ? "default" : rollup.running === 0 ? "secondary" : "outline"}
                class="tabular-nums"
              >
                {rollup.running}/{rollup.total} running
              </Badge>
            </div>
            {#if group.subtitle || group.projectPath}
              <p class="mt-1 truncate text-sm text-muted-foreground">
                {#if group.projectPath}
                  <span title={group.projectPath}>{group.projectPath}</span>
                {:else}
                  {group.subtitle}
                {/if}
              </p>
            {/if}
          </div>
          <ChevronDown
            class="mt-1 h-5 w-5 shrink-0 text-muted-foreground transition-transform {open
              ? 'rotate-180'
              : ''}"
          />
        </div>

        <div class="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
          {#if rollup.totalCpu > 0}
            <span class="inline-flex items-center gap-1">
              <Cpu class="h-3 w-3" />
              {fmtPercent(rollup.totalCpu)} CPU
            </span>
          {/if}
          {#if rollup.totalMemory > 0}
            <span class="inline-flex items-center gap-1">
              <MemoryStick class="h-3 w-3" />
              {fmtBytes(rollup.totalMemory)}
            </span>
          {/if}
          {#if group.images.length > 0}
            <span class="truncate">
              Images:
              {group.images.slice(0, 3).join(" · ")}
              {#if group.images.length > 3}
                +{group.images.length - 3} more
              {/if}
            </span>
          {/if}
          {#if group.networks.length > 0}
            <span class="truncate">
              Networks: {group.networks.slice(0, 2).join(", ")}
            </span>
          {/if}
        </div>
      </CardHeader>
    </CollapsibleTrigger>

    <CollapsibleContent>
      <CardContent class="divider-edge-t divider-edge-full space-y-3 pt-4">
        <div class="flex flex-wrap gap-2">
          {#if group.deployment}
            {#if group.deployment.status === "running"}
              <Button
                variant="outline"
                size="sm"
                onclick={() => onStopDeployment?.(group.deployment!.id)}
              >
                <Square class="mr-1 h-4 w-4" />
                Stop deployment
              </Button>
            {:else}
              <Button
                variant="default"
                size="sm"
                onclick={() => onStartDeployment?.(group.deployment!.id)}
              >
                <Play class="mr-1 h-4 w-4" />
                Start deployment
              </Button>
            {/if}
          {/if}
          {#if hasStopped && group.containers.length > 1}
            <Button variant="outline" size="sm" onclick={startAll}>
              <Play class="mr-1 h-4 w-4" />
              Start all
            </Button>
          {/if}
          {#if hasRunning && group.containers.length > 1}
            <Button variant="outline" size="sm" onclick={stopAll}>
              <Square class="mr-1 h-4 w-4" />
              Stop all
            </Button>
          {/if}
        </div>

        {#if group.containers.length === 0}
          <p class="text-sm text-muted-foreground italic">
            No containers linked. Start the deployment to create one.
          </p>
        {:else}
          <div class="space-y-2">
            {#each group.containers as container (container.id)}
              <ContainerRow
                {container}
                {onStart}
                {onStop}
                {onRemove}
              />
            {/each}
          </div>
        {/if}
      </CardContent>
    </CollapsibleContent>
  </Card>
</Collapsible>
