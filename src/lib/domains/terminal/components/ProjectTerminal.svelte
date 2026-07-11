<script lang="ts">
  import { logger } from "$lib/domains/shared";
  import { onMount, onDestroy } from "svelte";
  import { terminalStore, terminalActions } from "../stores/terminalStore";
  import { defaultTerminalConfig } from "../config/defaultTerminalConfig";
  import TerminalWorkspace from "./TerminalWorkspace.svelte";
  import type { TerminalConfig } from "../types";
  import {
    migratedProjectTabFields,
    projectTabNeedsMigration,
  } from "../utils/resolveSessionSettings";

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

    return terminalActions.createTabWithProcess({
      title: `${projectName} Terminal ${tabNumber}`,
      workingDirectory: projectPath,
      shell: actualShellCommand,
      resourceName: "project",
      resourceId: projectId,
    });
  }

  onMount(() => {
    log.info("ProjectTerminal mounted", { projectId });
    terminalActions.cleanupStaleData();

    for (const tab of $terminalStore.tabs) {
      if (
        projectTabNeedsMigration(
          tab,
          projectId,
          projectPath,
          settings.defaultShell,
        )
      ) {
        terminalActions.updateTab(
          tab.id,
          migratedProjectTabFields(projectPath, settings.defaultShell),
        );
      }
    }

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
  <div class="divider-edge-b divider-edge-full bg-card px-4 py-2">
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
      onNewTab={createNewTerminalTab}
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
