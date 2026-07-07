<script lang="ts">
  import DashboardTab from "./DashboardTab.svelte";
  import CleanupTab from "./CleanupTab.svelte";
  import ProjectsTab from "./ProjectsTab.svelte";
  import DevToolsTab from "./DevToolsTab.svelte";
  import HistoryTab from "./HistoryTab.svelte";
  import ProtectedTab from "./ProtectedTab.svelte";
  import SettingsTab from "./SettingsTab.svelte";
  import { Tabs, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
  import { HardDrive } from "@lucide/svelte";

  type Tab = "dashboard" | "cleanup" | "projects" | "devtools" | "history" | "protected" | "settings";
  const tabs: Tab[] = ["dashboard", "cleanup", "projects", "devtools", "history", "protected", "settings"];

  const TAB_LABELS: Record<Tab, string> = {
    dashboard: "Dashboard",
    cleanup: "Cleanup",
    projects: "Projects",
    devtools: "Dev tools",
    history: "History",
    protected: "Protected",
    settings: "Settings",
  };

  let tab = $state<Tab>("dashboard");

  // Deep-link request from the dashboard: switch tab + optionally auto-scan a path.
  let pending = $state<{ tab: Tab; path?: string; seq: number } | null>(null);
  let seq = 0;

  function go(target: "cleanup" | "projects" | "devtools", path?: string) {
    tab = target;
    seq += 1;
    pending = { tab: target, path, seq };
  }
</script>

<div class="mx-auto max-w-5xl px-6 py-8">
  <header class="mb-8">
    <div class="mb-2 flex items-center gap-2.5">
      <div class="flex h-6 w-6 items-center justify-center rounded-md bg-primary text-primary-foreground">
        <HardDrive class="size-3.5" />
      </div>
      <h1 class="text-lg font-semibold tracking-tight text-foreground">Disk Utility</h1>
    </div>
    <p class="max-w-2xl text-sm text-muted-foreground">
      Scans propose. You review. Nothing deletes without your tick — files go to the Recycle Bin; dev tools use their own remove commands.
    </p>
  </header>

  <Tabs value={tab} onValueChange={(v) => (tab = v as Tab)} class="mb-8">
    <TabsList>
      {#each tabs as t (t)}
        <TabsTrigger value={t}>{TAB_LABELS[t]}</TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>

  <div class={tab === "dashboard" ? "" : "hidden"}>
    <DashboardTab active={tab === "dashboard"} {go} />
  </div>
  <div class={tab === "cleanup" ? "" : "hidden"}>
    <CleanupTab {pending} />
  </div>
  <div class={tab === "projects" ? "" : "hidden"}>
    <ProjectsTab {pending} />
  </div>
  <div class={tab === "devtools" ? "" : "hidden"}>
    <DevToolsTab {pending} />
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
