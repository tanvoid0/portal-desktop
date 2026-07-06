<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AuditEntry } from "../types";
  import { fmtBytes, fmtDate } from "../utils";

  let { active }: { active: boolean } = $props();

  let entries = $state<AuditEntry[]>([]);
  let status = $state("Loading…");

  const btnGhostSm =
    "inline-flex items-center justify-center h-7 px-2.5 rounded-md text-xs font-medium text-neutral-300 border border-neutral-800 bg-neutral-950 transition-colors hover:bg-neutral-900 hover:text-white disabled:opacity-40 disabled:pointer-events-none";

  async function load() {
    status = "Loading…";
    try {
      const log = await invoke<AuditEntry[]>("get_audit_log");
      entries = log;
      status = log.length === 0 ? "No actions recorded yet." : `${log.length} action(s) recorded.`;
    } catch (e) {
      status = `Failed to load history: ${String(e)}`;
    }
  }

  let lastActive = false;
  $effect(() => {
    if (active && !lastActive) void load();
    lastActive = active;
  });

  const hstats = $derived.by(() => {
    const moved = entries.filter((e) => e.status === "moved");
    return {
      reclaimed: moved.reduce((a, e) => a + e.sizeBytes, 0),
      movedN: moved.length,
      failedN: entries.length - moved.length,
    };
  });

  async function openBin() {
    try {
      await invoke("open_recycle_bin");
    } catch (e) {
      status = String(e);
    }
  }
</script>

<div class="mb-4 grid grid-cols-3 gap-3">
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Total reclaimed</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight text-emerald-400">{fmtBytes(hstats.reclaimed)}</div>
  </div>
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Moved</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight text-white">{hstats.movedN}</div>
    <div class="mt-1 text-xs text-neutral-600">to Recycle Bin</div>
  </div>
  <div class="rounded-xl border border-neutral-800 bg-neutral-950/60 px-4 py-3.5">
    <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Failed</div>
    <div class="text-2xl font-semibold tabular-nums tracking-tight {hstats.failedN ? 'text-red-400' : 'text-white'}">{hstats.failedN}</div>
  </div>
</div>

<div class="mb-4 flex items-center gap-3">
  <div class="text-sm text-neutral-500">{status}</div>
  <button onclick={load} class="ml-auto {btnGhostSm}">Refresh</button>
  <button onclick={openBin} class={btnGhostSm}>Open Recycle Bin to undo</button>
</div>

<div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
  <table class="w-full text-sm">
    <thead class="bg-neutral-900/50 text-left text-xs uppercase tracking-wide text-neutral-500">
      <tr>
        <th class="px-3 py-2.5 font-medium">When</th>
        <th class="px-3 py-2.5 font-medium">Path</th>
        <th class="px-3 py-2.5 font-medium">Kind</th>
        <th class="px-3 py-2.5 text-right font-medium">Size</th>
        <th class="px-3 py-2.5 font-medium">Status</th>
      </tr>
    </thead>
    <tbody>
      {#each entries as e (e.id)}
        <tr class="border-t border-neutral-900 transition-colors hover:bg-neutral-900/40">
          <td class="whitespace-nowrap px-3 py-2.5 text-neutral-500">{fmtDate(e.ts)}</td>
          <td class="break-all px-3 py-2.5 font-mono text-xs text-neutral-300">{e.path}</td>
          <td class="px-3 py-2.5 text-neutral-400">{e.kind}</td>
          <td class="whitespace-nowrap px-3 py-2.5 text-right text-neutral-300">{fmtBytes(e.sizeBytes)}</td>
          <td class="px-3 py-2.5">
            <span class="inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium {e.status === 'moved' ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-400' : 'border-red-500/20 bg-red-500/10 text-red-400'}">{e.status}</span>
          </td>
        </tr>
      {/each}
      {#if entries.length === 0}
        <tr><td colspan="5" class="px-3 py-10 text-center text-neutral-600">Nothing here yet. Quarantined items will be logged.</td></tr>
      {/if}
    </tbody>
  </table>
</div>
