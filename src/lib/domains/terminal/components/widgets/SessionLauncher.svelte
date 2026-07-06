<script lang="ts">
  import { onMount } from "svelte";
  import { createProjectsQuery } from "$lib/domains/projects/queries/projectQueries";
  import {
    containers,
    deploymentActions,
    isDockerOffline,
  } from "$lib/domains/deployments/stores/deploymentStore";
  import { terminalActions } from "../../stores/terminalStore";
  import { defaultTerminalConfig } from "../../config/defaultTerminalConfig";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { FolderOpen, Container, Terminal } from "@lucide/svelte";
  import DockerStatusBanner from "$lib/domains/deployments/components/DockerStatusBanner.svelte";

  interface Props {
    onSessionOpened?: (tabId: string) => void;
  }

  let { onSessionOpened }: Props = $props();

  const projectsQuery = createProjectsQuery();
  const projects = $derived(projectsQuery.data ?? []);
  let containerList = $derived($containers);
  let dockerOffline = $derived($isDockerOffline);

  onMount(async () => {
    await deploymentActions.loadContainers();
  });

  function openProjectTerminal(project: {
    id: string;
    name: string;
    path: string;
  }) {
    const tabId = terminalActions.createTab({
      title: `${project.name} Terminal`,
      type: "terminal",
      workingDirectory: project.path,
      shell: defaultTerminalConfig.defaultShell,
      icon: "💻",
      closable: true,
      resourceName: "project",
      resourceId: project.id,
    });
    terminalActions.createProcess({
      tabId,
      command: defaultTerminalConfig.defaultShell,
      workingDirectory: project.path,
      environment: {},
      status: "running",
    });
    terminalActions.setActiveTab(tabId);
    onSessionOpened?.(tabId);
  }

  function openContainerTerminal(containerId: string, containerName: string) {
    const shell = navigator.userAgent.includes("Windows")
      ? "cmd.exe"
      : "bash";
    const tabId = terminalActions.createTab({
      title: `Container: ${containerName}`,
      type: "terminal",
      workingDirectory: defaultTerminalConfig.workingDirectory,
      shell: `docker exec -it ${containerId} ${shell}`,
      icon: "🐳",
      closable: true,
      resourceName: "container",
      resourceId: containerId,
    });
    terminalActions.createProcess({
      tabId,
      command: `docker exec -it ${containerId} ${shell}`,
      workingDirectory: defaultTerminalConfig.workingDirectory,
      environment: {},
      status: "running",
    });
    terminalActions.setActiveTab(tabId);
    onSessionOpened?.(tabId);
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="border-b border-gray-700 p-2">
    <div class="text-sm font-medium text-gray-200">Session Launcher</div>
    <p class="text-xs text-gray-400">Open terminals for projects or containers</p>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-3 space-y-4">
    <section>
      <div class="mb-2 flex items-center gap-2 text-xs font-medium uppercase text-gray-400">
        <FolderOpen class="h-3.5 w-3.5" />
        Projects
      </div>
      {#if projects.length === 0}
        <p class="text-xs text-gray-500">No projects found</p>
      {:else}
        <div class="space-y-1">
          {#each projects as project (project.id)}
            <Button
              variant="ghost"
              size="sm"
              class="h-auto w-full justify-start py-2 text-left"
              onclick={() => openProjectTerminal(project)}
            >
              <Terminal class="mr-2 h-3.5 w-3.5 shrink-0" />
              <span class="truncate">{project.name}</span>
            </Button>
          {/each}
        </div>
      {/if}
    </section>

    <section>
      <div class="mb-2 flex items-center gap-2 text-xs font-medium uppercase text-gray-400">
        <Container class="h-3.5 w-3.5" />
        Containers
      </div>
      <DockerStatusBanner
        onReady={async () => {
          await deploymentActions.loadContainers();
        }}
      />
      {#if containerList.length === 0}
        <p class="text-xs text-gray-500">
          {dockerOffline ? "Docker is offline" : "No containers found"}
        </p>
      {:else}
        <div class="space-y-1">
          {#each containerList as container (container.id)}
            <div class="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                class="h-auto flex-1 justify-start py-2 text-left"
                disabled={container.status !== "running"}
                onclick={() =>
                  openContainerTerminal(container.id, container.name)}
              >
                <Terminal class="mr-2 h-3.5 w-3.5 shrink-0" />
                <span class="truncate">{container.name}</span>
              </Button>
              <Badge variant="outline" class="text-xs shrink-0">
                {container.status}
              </Badge>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>
