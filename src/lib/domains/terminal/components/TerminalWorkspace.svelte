<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { terminalStore, terminalActions } from "../stores/terminalStore";
  import { commandBlockStore } from "../stores/commandBlockStore";
  import { defaultTerminalConfig } from "../config/defaultTerminalConfig";
  import {
    loadWidgetRailState,
    saveWidgetRailState,
    toggleWidget,
    type WidgetId,
    type WidgetRailState,
  } from "../stores/widgetRailStore";
  import TabContainer from "./TabContainer.svelte";
  import TerminalSession from "./core/TerminalSession.svelte";
  import CommandBlocksPanel from "./widgets/CommandBlocksPanel.svelte";
  import AIAssistantPanel from "./widgets/AIAssistantPanel.svelte";
  import NotesPanel from "./widgets/NotesPanel.svelte";
  import SessionLauncher from "./widgets/SessionLauncher.svelte";
  import CommandHistoryPanel from "./widgets/CommandHistoryPanel.svelte";
  import CommandPalette from "./CommandPalette.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    ResizablePaneGroup,
    ResizablePane,
    ResizableHandle,
  } from "$lib/components/ui/resizable";
  import {
    Blocks,
    Bot,
    StickyNote,
    Rocket,
    History,
    PanelRight,
  } from "@lucide/svelte";
  import type { TerminalTab } from "../stores/terminalStore";
  import type { TerminalConfig } from "../types";

  interface Props {
    settings?: TerminalConfig;
    tabFilter?: (tab: TerminalTab) => boolean;
    showLauncher?: boolean;
    showHistory?: boolean;
    autoCreateTab?: boolean;
  }

  let {
    settings = defaultTerminalConfig,
    tabFilter = (tab) => !tab.resourceName && !tab.resourceId,
    showLauncher = true,
    showHistory = true,
    autoCreateTab = true,
  }: Props = $props();

  let widgetRail = $state<WidgetRailState>(loadWidgetRailState());
  let deepLinkHandled = $state(false);

  const tabs = $derived($terminalStore.tabs.filter(tabFilter));
  const activeTabId = $derived($terminalStore.activeTabId);

  const widgetToggles: Array<{ id: WidgetId; label: string; icon: typeof Blocks }> =
    [
      { id: "blocks", label: "Blocks", icon: Blocks },
      { id: "ai", label: "AI", icon: Bot },
      { id: "notes", label: "Notes", icon: StickyNote },
      ...(showLauncher
        ? [{ id: "launcher" as WidgetId, label: "Launcher", icon: Rocket }]
        : []),
      ...(showHistory
        ? [{ id: "history" as WidgetId, label: "History", icon: History }]
        : []),
    ];

  function createNewTerminalTab(shellCommand?: string) {
    const tabNumber = tabs.length + 1;
    const actualShell = shellCommand || settings.defaultShell;
    const tabId = terminalActions.createTab({
      title: `Terminal ${tabNumber}`,
      type: "terminal",
      workingDirectory: settings.workingDirectory,
      shell: actualShell,
      icon: "💻",
      closable: true,
    });
    terminalActions.createProcess({
      tabId,
      command: actualShell,
      workingDirectory: settings.workingDirectory,
      environment: {},
      status: "running",
    });
    return tabId;
  }

  function handleWidgetToggle(widget: WidgetId) {
    widgetRail = toggleWidget(widgetRail, widget);
    saveWidgetRailState(widgetRail);
  }

  function handleRerun(tabId: string, command: string) {
    sessionRefs[tabId]?.rerunCommand(command);
  }

  async function handleDeepLinks() {
    if (deepLinkHandled) return;
    deepLinkHandled = true;

    const params = page.url.searchParams;
    const command = params.get("command");
    const containerId = params.get("container");
    const projectId = params.get("project");

    if (projectId) {
      const existing = tabs.find(
        (t) => t.resourceName === "project" && t.resourceId === projectId,
      );
      if (existing) {
        terminalActions.setActiveTab(existing.id);
      }
    }

    if (containerId) {
      const shell = navigator.userAgent.includes("Windows")
        ? "cmd.exe"
        : "bash";
      const tabId = terminalActions.createTab({
        title: `Container ${containerId.slice(0, 8)}`,
        type: "terminal",
        workingDirectory: settings.workingDirectory,
        shell: `docker exec -it ${containerId} ${shell}`,
        icon: "🐳",
        closable: true,
        resourceName: "container",
        resourceId: containerId,
      });
      terminalActions.createProcess({
        tabId,
        command: `docker exec -it ${containerId} ${shell}`,
        workingDirectory: settings.workingDirectory,
        environment: {},
        status: "running",
      });
      terminalActions.setActiveTab(tabId);
    }

    if (command && activeTabId) {
      // Pre-fill handled by session on next tick — store for injection
      pendingCommand = command;
    }
  }

  let pendingCommand = $state<string | null>(null);
  let sessionRefs = $state<Record<string, ReturnType<typeof TerminalSession> | null>>({});

  onMount(() => {
    commandBlockStore.startShellIntegrationListener();

    if (autoCreateTab && tabs.length === 0) {
      createNewTerminalTab();
    } else if (
      tabs.length > 0 &&
      (!activeTabId || !tabs.some((tab) => tab.id === activeTabId))
    ) {
      terminalActions.setActiveTab(tabs[0].id);
    }

    handleDeepLinks();
  });

  $effect(() => {
    saveWidgetRailState(widgetRail);
  });

  $effect(() => {
    if (
      tabs.length > 0 &&
      (!activeTabId || !tabs.some((tab) => tab.id === activeTabId))
    ) {
      terminalActions.setActiveTab(tabs[0].id);
    }
  });
</script>

<TabContainer
  onNewTab={createNewTerminalTab}
  {tabFilter}
  className="terminal-workspace h-full bg-background"
>
  {#if tabs.length === 0}
    <div class="flex h-full items-center justify-center">
      <Button onclick={() => createNewTerminalTab()}>Create New Tab</Button>
    </div>
  {:else}
    {#each tabs as tab (tab.id)}
      <div
        class="h-full w-full"
        style:display={tab.id === activeTabId ? "block" : "none"}
      >
        <div class="flex h-full flex-col">
          <div class="flex items-center justify-end gap-1 border-b border-border bg-card px-2 py-1">
            {#each widgetToggles as toggle (toggle.id)}
              <Button
                variant={widgetRail.activeWidgets.includes(toggle.id)
                  ? "secondary"
                  : "ghost"}
                size="sm"
                class="h-7 gap-1 px-2 text-xs"
                onclick={() => handleWidgetToggle(toggle.id)}
              >
                <toggle.icon class="h-3.5 w-3.5" />
                {toggle.label}
              </Button>
            {/each}
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2"
              onclick={() => {
                widgetRail = {
                  ...widgetRail,
                  open: !widgetRail.open,
                };
              }}
              title="Toggle widget rail"
            >
              <PanelRight class="h-3.5 w-3.5" />
            </Button>
          </div>

          <div class="min-h-0 flex-1">
            {#if widgetRail.open && widgetRail.activeWidgets.length > 0}
              <ResizablePaneGroup direction="horizontal" class="h-full">
                <ResizablePane defaultSize={70} minSize={40} class="min-h-0">
                  <div class="h-full min-h-0">
                    <TerminalSession
                    bind:this={sessionRefs[tab.id]}
                    tabId={tab.id}
                    initialCommand={tab.id === activeTabId ? pendingCommand ?? undefined : undefined}
                    settings={{
                      ...settings,
                      defaultShell: tab.shell || settings.defaultShell,
                      workingDirectory:
                        tab.workingDirectory || settings.workingDirectory,
                    }}
                  />
                  </div>
                </ResizablePane>
                <ResizableHandle withHandle />
                <ResizablePane defaultSize={30} minSize={20}>
                  <div class="flex h-full flex-col divide-y divide-border overflow-hidden">
                    {#if widgetRail.activeWidgets.includes("blocks")}
                      <div class="min-h-0 flex-1">
                        <CommandBlocksPanel
                          tabId={tab.id}
                          onRerun={(cmd) => handleRerun(tab.id, cmd)}
                          onExplain={(block) =>
                            sessionRefs[tab.id]?.explainError(block)}
                        />
                      </div>
                    {/if}
                    {#if widgetRail.activeWidgets.includes("ai")}
                      <div class="min-h-0 flex-1">
                        <AIAssistantPanel
                          tabId={tab.id}
                          shell={tab.shell || settings.defaultShell}
                          workingDirectory={
                            tab.workingDirectory || settings.workingDirectory
                          }
                          onRunCommand={(cmd) => handleRerun(tab.id, cmd)}
                        />
                      </div>
                    {/if}
                    {#if widgetRail.activeWidgets.includes("notes")}
                      <div class="min-h-0 flex-1">
                        <NotesPanel tabId={tab.id} />
                      </div>
                    {/if}
                    {#if showLauncher && widgetRail.activeWidgets.includes("launcher")}
                      <div class="min-h-0 flex-1">
                        <SessionLauncher
                          onSessionOpened={(id) =>
                            terminalActions.setActiveTab(id)}
                        />
                      </div>
                    {/if}
                    {#if showHistory && widgetRail.activeWidgets.includes("history")}
                      <div class="min-h-0 flex-1">
                        <CommandHistoryPanel tabId={tab.id} />
                      </div>
                    {/if}
                  </div>
                </ResizablePane>
              </ResizablePaneGroup>
            {:else}
              <TerminalSession
                bind:this={sessionRefs[tab.id]}
                tabId={tab.id}
                settings={{
                  ...settings,
                  defaultShell: tab.shell || settings.defaultShell,
                  workingDirectory:
                    tab.workingDirectory || settings.workingDirectory,
                }}
              />
            {/if}
          </div>
        </div>
      </div>
    {/each}
  {/if}
</TabContainer>

{#if activeTabId}
  <CommandPalette
    tabId={activeTabId}
    onKillProcess={() => {}}
    onClearTerminal={() => {}}
    onRerunCommand={(cmd) => handleRerun(activeTabId, cmd)}
  />
{/if}
