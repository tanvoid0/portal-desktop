<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import type { DockerContainer } from "../types";
  import { fmtBytes, fmtPercent, isContainerRunning } from "../utils/format";
  import { Play, Square, Trash2, Cpu, MemoryStick } from "@lucide/svelte";

  interface Props {
    container: DockerContainer;
    onStart?: (id: string) => void;
    onStop?: (id: string) => void;
    onRemove?: (id: string) => void;
  }

  let { container, onStart, onStop, onRemove }: Props = $props();

  const running = $derived(isContainerRunning(container.status));
  const stats = $derived(container.resourceStats);

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
        .map((p: any) => {
          if (typeof p === "string") return p;
          return `${p.hostPort || ""}:${p.containerPort || ""}`;
        })
        .join(", ");
    }
    if (typeof container.ports === "string") {
      return container.ports || "—";
    }
    return "—";
  }
</script>

<Card class="w-full transition-shadow hover:shadow-md">
  <CardContent class="p-4">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0 flex-1">
        <div class="mb-2 flex items-center gap-2">
          <h3 class="truncate text-base font-semibold" title={container.name}>
            {container.name}
          </h3>
          <Badge class={getStatusColor()}>
            {getStatusLabel()}
          </Badge>
        </div>

        <div class="mb-3 grid grid-cols-2 gap-x-4 gap-y-1 text-sm text-muted-foreground">
          <div class="col-span-2 flex items-center gap-2">
            <span class="shrink-0 font-medium text-foreground/70">Image</span>
            <span class="truncate font-mono text-xs" title={container.image}>
              {container.image}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span class="shrink-0 font-medium text-foreground/70">Ports</span>
            <span class="truncate font-mono text-xs">{formatPorts()}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="shrink-0 font-medium text-foreground/70">ID</span>
            <span class="truncate font-mono text-xs" title={container.id}>
              {container.id.substring(0, 12)}
            </span>
          </div>
          {#if container.createdAt}
            {@const createdDate =
              container.createdAt instanceof Date
                ? container.createdAt
                : new Date(container.createdAt)}
            {#if !isNaN(createdDate.getTime())}
              <div class="col-span-2 flex items-center gap-2">
                <span class="shrink-0 font-medium text-foreground/70"
                  >Created</span
                >
                <span class="text-xs">
                  {createdDate.toLocaleDateString()}
                  {createdDate.toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </span>
              </div>
            {/if}
          {/if}
        </div>

        {#if running && stats}
          <div class="space-y-2 rounded-md border bg-muted/30 p-2.5">
            <div class="space-y-1">
              <div class="flex items-center justify-between text-xs">
                <span class="flex items-center gap-1 text-muted-foreground">
                  <Cpu class="h-3 w-3" /> CPU
                </span>
                <span class="tabular-nums font-medium"
                  >{fmtPercent(stats.cpuPercent)}</span
                >
              </div>
              <Progress
                value={Math.min(stats.cpuPercent, 100)}
                class="h-1 [&>[data-slot=progress-indicator]]:bg-emerald-500"
              />
            </div>
            <div class="space-y-1">
              <div class="flex items-center justify-between text-xs">
                <span class="flex items-center gap-1 text-muted-foreground">
                  <MemoryStick class="h-3 w-3" /> Memory
                </span>
                <span class="tabular-nums font-medium">
                  {fmtBytes(stats.memoryBytes)}
                  {#if stats.memoryLimitBytes}
                    / {fmtBytes(stats.memoryLimitBytes)}
                  {/if}
                  · {fmtPercent(stats.memoryPercent)}
                </span>
              </div>
              <Progress
                value={Math.min(stats.memoryPercent, 100)}
                class="h-1 [&>[data-slot=progress-indicator]]:bg-blue-500"
              />
            </div>
          </div>
        {:else if running}
          <p class="text-xs text-muted-foreground italic">
            Resource stats unavailable
          </p>
        {/if}
      </div>

      <div class="flex shrink-0 flex-col gap-2">
        {#if running}
          <Button
            variant="outline"
            size="sm"
            onclick={() => onStop?.(container.id)}
          >
            <Square class="mr-1 h-4 w-4" />
            Stop
          </Button>
        {:else}
          <Button
            variant="default"
            size="sm"
            onclick={() => onStart?.(container.id)}
          >
            <Play class="mr-1 h-4 w-4" />
            Start
          </Button>
        {/if}
        <Button
          variant="destructive"
          size="sm"
          onclick={() => onRemove?.(container.id)}
        >
          <Trash2 class="mr-1 h-4 w-4" />
          Remove
        </Button>
      </div>
    </div>
  </CardContent>
</Card>
