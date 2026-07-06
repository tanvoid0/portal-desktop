<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  let { active }: { active: boolean } = $props();

  let paths = $state<string[]>([]);
  let input = $state("");
  let status = $state("");

  const btnPrimary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-white text-black text-sm font-medium transition-colors hover:bg-neutral-200 disabled:opacity-40 disabled:pointer-events-none";
  const btnSecondary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-neutral-900 text-neutral-200 text-sm font-medium border border-neutral-800 transition-colors hover:bg-neutral-800 hover:border-neutral-700 disabled:opacity-40 disabled:pointer-events-none";
  const inputCls =
    "h-9 px-3 rounded-md bg-neutral-950 border border-neutral-800 text-sm text-neutral-100 placeholder:text-neutral-600 transition-colors focus:outline-none focus:border-neutral-600";

  async function load() {
    try {
      paths = await invoke<string[]>("list_protected");
    } catch (e) {
      status = `Failed to load: ${String(e)}`;
    }
  }

  let lastActive = false;
  $effect(() => {
    if (active && !lastActive) void load();
    lastActive = active;
  });

  async function add(path: string) {
    const p = path.trim();
    if (!p) return;
    try {
      await invoke("add_protected", { path: p });
      input = "";
      status = `Protected: ${p}`;
      load();
    } catch (e) {
      status = `Add failed: ${String(e)}`;
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") add(dir);
  }

  async function remove(path: string) {
    try {
      await invoke("remove_protected", { path });
      status = `Removed: ${path}`;
      load();
    } catch (e) {
      status = `Remove failed: ${String(e)}`;
    }
  }
</script>

<p class="mb-4 max-w-2xl text-sm text-neutral-500">
  Paths listed here are never proposed for cleanup — anything under them is skipped during scans. OS
  and program folders are already protected by default.
</p>

<div class="mb-4 flex items-center gap-2">
  <button onclick={pickFolder} class={btnSecondary}>Add folder…</button>
  <input bind:value={input} onkeydown={(e) => e.key === "Enter" && add(input)} placeholder="C:\Users\you\Projects" class="flex-1 {inputCls}" />
  <button onclick={() => add(input)} disabled={!input.trim()} class={btnPrimary}>Protect</button>
</div>

{#if status}<div class="mb-4 text-sm text-neutral-500">{status}</div>{/if}

<div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
  <table class="w-full text-sm">
    <thead class="bg-neutral-900/50 text-left text-xs uppercase tracking-wide text-neutral-500">
      <tr><th class="px-3 py-2.5 font-medium">Protected path</th><th class="w-24 px-3 py-2.5"></th></tr>
    </thead>
    <tbody>
      {#each paths as p (p)}
        <tr class="border-t border-neutral-900 transition-colors hover:bg-neutral-900/40">
          <td class="break-all px-3 py-2.5 font-mono text-xs text-neutral-300">{p}</td>
          <td class="px-3 py-2.5 text-right">
            <button onclick={() => remove(p)} class="inline-flex h-7 items-center rounded-md border border-neutral-800 bg-neutral-950 px-2.5 text-xs font-medium text-neutral-400 transition-colors hover:border-red-600 hover:bg-red-600 hover:text-white">Remove</button>
          </td>
        </tr>
      {/each}
      {#if paths.length === 0}
        <tr><td colspan="2" class="px-3 py-10 text-center text-neutral-600">No custom protected paths. Add folders you never want touched.</td></tr>
      {/if}
    </tbody>
  </table>
</div>
