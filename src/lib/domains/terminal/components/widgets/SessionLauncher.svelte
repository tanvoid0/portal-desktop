<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { projectTerminalHref } from "../../navigation";
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
  import { FolderOpen, Container } from "@lucide/svelte";
  import ShellIcon from "../ShellIcon.svelte";
  import { TERMINAL_ICONS } from "../../utils/shellIcons";
  import DockerStatusBanner from "$lib/domains/deployments/components/DockerStatusBanner.svelte";

  interface Props {
    /** Called when a container tab is opened in the global terminal workspace. */
    onContainerOpened?: (tabId: string) => void;
  }

  let { onContainerOpened }: Props = $props();

  const projectsQuery = createProjectsQuery();
  const projects = $derived(projectsQuery.data ?? []);
  let containerList = $derived($containers);
  let dockerOffline = $derived($isDockerOffline);

  onMount(async () => {
    await deploymentActions.loadContainers();
  });

  function openProjectTerminal(project: { id: string; name: string; path: string }) {
    goto(projectTerminalHref(project.id));
  }

  function openContainerTerminal(containerId: string, containerName: string) {
    const shell = navigator.userAgent.includes("Windows")
      ? "cmd.exe"
      : "bash";
    const command = `docker exec -it ${containerId} ${shell}`;
    const tabId = terminalActions.createTabWithProcess({
      title: `Container: ${containerName}`,
      workingDirectory: defaultTerminalConfig.workingDirectory,
      shell: command,
      icon: TERMINAL_ICONS.container,
      resourceName: "container",
      resourceId: containerId,
    });
    terminalActions.setActiveTab(tabId);
    onContainerOpened?.(tabId);
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="divider-edge-b divider-edge-full p-2">
    <div class="text-sm font-medium text-foreground">Session Launcher</div>
    <p class="text-xs text-muted-foreground">
      Open project terminals in the project workspace, or attach to containers here
    </p>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-3 space-y-4">
    <section>
      <div class="mb-2 flex items-center gap-2 text-xs font-medium uppercase text-muted-foreground">
        <FolderOpen class="h-3.5 w-3.5" />
        Projects
      </div>
      {#if projects.length === 0}
        <p class="text-xs text-muted-foreground">No projects found</p>
      {:else}
        <div class="space-y-1">
          {#each projects as project (project.id)}
            <Button
              variant="ghost"
              size="sm"
              class="h-auto w-full justify-start py-2 text-left"
              onclick={() => openProjectTerminal(project)}
            >
              <ShellIcon icon="codicon:terminal" class="mr-2" />
              <span class="truncate">{project.name}</span>
            </Button>
          {/each}
        </div>
      {/if}
    </section>

    <section>
      <div class="mb-2 flex items-center gap-2 text-xs font-medium uppercase text-muted-foreground">
        <Container class="h-3.5 w-3.5" />
        Containers
      </div>
      <DockerStatusBanner
        onReady={async () => {
          await deploymentActions.loadContainers();
        }}
      />
      {#if containerList.length === 0}
        <p class="text-xs text-muted-foreground">
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
                <ShellIcon icon={TERMINAL_ICONS.container} class="mr-2" />
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
