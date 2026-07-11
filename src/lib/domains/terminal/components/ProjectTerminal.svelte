<script lang="ts">
  import { logger } from "$lib/domains/shared";
  import { onMount, onDestroy } from "svelte";
  import { terminalStore, terminalActions } from "../stores/terminalStore";
  import { defaultTerminalConfig } from "../config/defaultTerminalConfig";
  import TerminalWorkspace from "./TerminalWorkspace.svelte";
  import type { TerminalConfig } from "../types";

  const log = logger.createScoped("ProjectTerminal");

  interface Props {
    projectId: string;
    projectName: string;
    projectPath: string;
    settings?: TerminalConfig;
  }

  let {
    projectId,
    projectName,
    projectPath,
    settings: providedSettings,
  }: Props = $props();

  const settings = $derived(
    providedSettings ?? {
      ...defaultTerminalConfig,
      workingDirectory: projectPath,
    },
  );

  const tabs = $derived(
    $terminalStore.tabs.filter(
      (tab) => tab.resourceName === "project" && tab.resourceId === projectId,
    ),
  );

  function createNewTerminalTab(shellCommand?: string) {
    log.debug("Creating new project terminal tab", { projectId });
    const tabNumber = tabs.length + 1;
    const actualShellCommand = shellCommand || settings.defaultShell;

    const tabId = terminalActions.createTab({
      title: `${projectName} Terminal ${tabNumber}`,
      type: "terminal",
      workingDirectory: projectPath,
      shell: actualShellCommand,
      icon: "💻",
      closable: true,
      resourceName: "project",
      resourceId: projectId,
    });

    terminalActions.createProcess({
      tabId,
      command: actualShellCommand,
      workingDirectory: projectPath,
      environment: {},
      status: "running",
    });

    return tabId;
  }

  onMount(() => {
    log.info("ProjectTerminal mounted", { projectId });
    terminalActions.cleanupStaleData();

    if (tabs.length === 0) {
      createNewTerminalTab();
    } else if (
      tabs.length > 0 &&
      (!$terminalStore.activeTabId ||
        !tabs.some((tab) => tab.id === $terminalStore.activeTabId))
    ) {
      terminalActions.setActiveTab(tabs[0].id);
    }
  });

  onDestroy(() => {
    log.info("ProjectTerminal destroyed", { projectId });
  });
</script>

<div class="project-terminal-container flex h-full w-full flex-col bg-background">
  <div class="border-b border-border bg-card px-4 py-2">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold text-foreground">
        {projectName} Terminal
      </h2>
      <span class="font-mono text-xs text-muted-foreground">{projectPath}</span>
    </div>
  </div>

  <div class="min-h-0 flex-1">
    <TerminalWorkspace
      {settings}
      showLauncher={false}
      showHistory={false}
      autoCreateTab={false}
      tabFilter={(tab) =>
        tab.resourceName === "project" && tab.resourceId === projectId}
    />
  </div>
</div>

<style>
  .project-terminal-container {
    background: hsl(var(--background));
  }
</style>
