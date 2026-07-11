<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import type { DockerContainer } from "../types";
  import {
    fmtBytes,
    fmtPercent,
    isContainerRunning,
  } from "../utils/format";
  import { resolveComposeService as getService } from "../utils/workloadGrouping";
  import ContainerCard from "./ContainerCard.svelte";
  import { Play, Square, Trash2, ChevronDown, Cpu, MemoryStick } from "@lucide/svelte";

  interface Props {
    container: DockerContainer;
    onStart?: (id: string) => void;
    onStop?: (id: string) => void;
    onRemove?: (id: string) => void;
  }

  let { container, onStart, onStop, onRemove }: Props = $props();

  let expanded = $state(false);

  const running = $derived(isContainerRunning(container.status));
  const stats = $derived(container.resourceStats);
  const displayName = $derived(getService(container) ?? container.name.replace(/^\//, ""));

  function getStatusColor(): string {
    if (running) {
      return "bg-green-100 text-green-800 border-green-200 dark:bg-green-950 dark:text-green-300";
    }
    const s = container.status?.toLowerCase() || "";
    if (s.includes("paused")) {
      return "bg-yellow-100 text-yellow-800 border-yellow-200";
    }
    return "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-800 dark:text-gray-300";
  }

  function getStatusLabel(): string {
    if (running) return "Running";
    const s = container.status?.toLowerCase() || "";
    if (s.includes("exited")) return "Exited";
    if (s.includes("paused")) return "Paused";
    if (s.includes("created")) return "Created";
    return "Stopped";
  }

  function formatPorts(): string {
    if (Array.isArray(container.ports)) {
      if (container.ports.length === 0) return "—";
      return container.ports
        .map((p: string | { hostPort?: number; containerPort?: number }) => {
          if (typeof p === "string") return p;
          return `${p.hostPort || ""}:${p.containerPort || ""}`;
        })
        .join(", ");
    }
    return "—";
  }
</script>

<div class="rounded-lg border border-border/60 bg-card shadow-sm transition-shadow hover:shadow-md">
  <div class="flex items-center gap-3 p-3">
    <Button
      variant="ghost"
      size="icon-sm"
      class="shrink-0"
      onclick={() => (expanded = !expanded)}
      aria-expanded={expanded}
      aria-label={expanded ? "Collapse container details" : "Expand container details"}
    >
      <ChevronDown
        class="h-4 w-4 transition-transform {expanded ? 'rotate-180' : ''}"
      />
    </Button>

    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-2">
        <span class="truncate font-medium" title={displayName}>{displayName}</span>
        <Badge class={getStatusColor()}>{getStatusLabel()}</Badge>
        {#if getService(container) && container.name.replace(/^\//, "") !== displayName}
          <span class="truncate text-xs text-muted-foreground" title={container.name}>
            {container.name.replace(/^\//, "")}
          </span>
        {/if}
      </div>
      <div class="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        <span class="truncate font-mono" title={container.image}>{container.image}</span>
        <span>{formatPorts()}</span>
        {#if running && stats}
          <span class="inline-flex items-center gap-1">
            <Cpu class="h-3 w-3" />
            {fmtPercent(stats.cpuPercent)}
          </span>
          <span class="inline-flex items-center gap-1">
            <MemoryStick class="h-3 w-3" />
            {fmtBytes(stats.memoryBytes)}
          </span>
        {/if}
      </div>
      {#if running && stats}
        <div class="mt-2 grid gap-1 sm:grid-cols-2">
          <Progress
            value={Math.min(stats.cpuPercent, 100)}
            class="h-1 [&>[data-slot=progress-indicator]]:bg-emerald-500"
          />
          <Progress
            value={Math.min(stats.memoryPercent, 100)}
            class="h-1 [&>[data-slot=progress-indicator]]:bg-blue-500"
          />
        </div>
      {/if}
    </div>

    <div class="flex shrink-0 gap-1">
      {#if running}
        <Button variant="outline" size="sm" onclick={() => onStop?.(container.id)}>
          <Square class="h-4 w-4" />
        </Button>
      {:else}
        <Button variant="default" size="sm" onclick={() => onStart?.(container.id)}>
          <Play class="h-4 w-4" />
        </Button>
      {/if}
      <Button variant="destructive" size="sm" onclick={() => onRemove?.(container.id)}>
        <Trash2 class="h-4 w-4" />
      </Button>
    </div>
  </div>

  {#if expanded}
    <div class="border-t p-3">
      <ContainerCard
        {container}
        {onStart}
        {onStop}
        {onRemove}
      />
    </div>
  {/if}
</div>
