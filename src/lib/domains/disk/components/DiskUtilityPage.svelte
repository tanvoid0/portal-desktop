<script lang="ts">
  import DashboardTab from "./DashboardTab.svelte";
  import CleanupTab from "./CleanupTab.svelte";
  import ProjectsTab from "./ProjectsTab.svelte";
  import HistoryTab from "./HistoryTab.svelte";
  import ProtectedTab from "./ProtectedTab.svelte";
  import SettingsTab from "./SettingsTab.svelte";

  type Tab = "dashboard" | "cleanup" | "projects" | "history" | "protected" | "settings";
  const tabs: Tab[] = ["dashboard", "cleanup", "projects", "history", "protected", "settings"];

  let tab = $state<Tab>("dashboard");

  // Deep-link request from the dashboard: switch tab + optionally auto-scan a path.
  let pending = $state<{ tab: Tab; path?: string; seq: number } | null>(null);
  let seq = 0;

  function go(target: "cleanup" | "projects", path?: string) {
    tab = target;
    seq += 1;
    pending = { tab: target, path, seq };
  }
</script>

<div class="mx-auto max-w-5xl px-6 py-8">
  <header class="mb-8">
    <div class="mb-2 flex items-center gap-2.5">
      <div class="flex h-6 w-6 items-center justify-center rounded-md bg-white">
        <span class="text-sm font-bold leading-none text-black">◑</span>
      </div>
      <h1 class="text-lg font-semibold tracking-tight">Disk Utility</h1>
    </div>
    <p class="max-w-2xl text-sm text-neutral-500">
      Scans propose. You review. Nothing deletes without your tick — items go to the Recycle Bin.
    </p>
  </header>

  <div class="mb-8 flex gap-1 border-b border-neutral-900">
    {#each tabs as t (t)}
      <button
        onclick={() => (tab = t)}
        class="relative px-3 py-2.5 text-sm capitalize transition-colors {tab === t
          ? 'text-white'
          : 'text-neutral-500 hover:text-neutral-300'}"
      >
        {t}
        {#if tab === t}
          <span class="absolute inset-x-0 -bottom-px h-px bg-white"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class={tab === "dashboard" ? "" : "hidden"}>
    <DashboardTab active={tab === "dashboard"} {go} />
  </div>
  <div class={tab === "cleanup" ? "" : "hidden"}>
    <CleanupTab {pending} />
  </div>
  <div class={tab === "projects" ? "" : "hidden"}>
    <ProjectsTab {pending} />
  </div>
  <div class={tab === "history" ? "" : "hidden"}>
    <HistoryTab active={tab === "history"} />
  </div>
  <div class={tab === "protected" ? "" : "hidden"}>
    <ProtectedTab active={tab === "protected"} />
  </div>
  <div class={tab === "settings" ? "" : "hidden"}>
    <SettingsTab />
  </div>
</div>
