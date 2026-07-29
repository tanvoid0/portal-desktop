<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Cpu, MemoryStick } from "@lucide/svelte";

  interface HostStats {
    cpuPercent: number;
    memoryUsedBytes: number;
    memoryTotalBytes: number;
  }

  let stats = $state<HostStats | null>(null);
  /** One failure is enough — outside the desktop shell it will never work. */
  let unavailable = $state(false);

  const memoryLabel = $derived(
    stats ? `${(stats.memoryUsedBytes / 1024 ** 3).toFixed(1)} GB` : "",
  );
  const cpuLabel = $derived(stats ? `${stats.cpuPercent.toFixed(1)}%` : "");
  const memoryTitle = $derived(
    stats
      ? `${memoryLabel} of ${(stats.memoryTotalBytes / 1024 ** 3).toFixed(1)} GB used`
      : "",
  );

  async function poll() {
    try {
      stats = await invoke<HostStats>("host_stats");
    } catch {
      unavailable = true;
    }
  }

  $effect(() => {
    if (unavailable) return;
    void poll();
    const timer = setInterval(poll, 3000);
    return () => clearInterval(timer);
  });
</script>

{#if stats}
  <div
    class="divider-edge-t divider-edge-full flex shrink-0 items-center justify-center gap-1.5 px-3 py-1.5"
  >
    <span
      class="inline-flex items-center gap-1 rounded-md bg-muted/60 px-1.5 py-0.5 text-[11px] tabular-nums text-muted-foreground"
      title={memoryTitle}
    >
      <MemoryStick class="h-3 w-3" />
      {memoryLabel}
    </span>
    <span
      class="inline-flex items-center gap-1 rounded-md bg-muted/60 px-1.5 py-0.5 text-[11px] tabular-nums text-muted-foreground"
      title="Host CPU load"
    >
      <Cpu class="h-3 w-3" />
      {cpuLabel}
    </span>
  </div>
{/if}
