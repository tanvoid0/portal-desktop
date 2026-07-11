<script lang="ts">
  import {
    terminalStore,
    terminalActions,
    tabCount,
  } from "../stores/terminalStore";
  import type { TerminalTab } from "../stores/terminalStore";
  import { Plus, X, ChevronDown } from "@lucide/svelte";
  import { TerminalService } from "../services/terminalService";
  import { Button } from "$lib/components/ui/button";
  import { onMount } from "svelte";
  import { logger } from "$lib/domains/shared";
  import ShellIcon from "./ShellIcon.svelte";
  import ShellProfileSelect from "./ShellProfileSelect.svelte";
  import { resolveShellIcon } from "../utils/shellIcons";

  const log = logger.createScoped("TabBar");

  // Props
  interface Props {
    tabs: TerminalTab[];
    onNewTab?: ((profileName?: string) => void) | null;
    showNewTabButton?: boolean;
    closable?: boolean;
    showProfileSelector?: boolean;
  }

  let {
    tabs,
    onNewTab = null,
    showNewTabButton = true,
    closable = true,
    showProfileSelector = true,
  }: Props = $props();

  // Terminal profile state
  let availableProfiles = $state<any[]>([]);
  let selectedProfile = $state("");
  let systemInfo = $state<any>(null);

  // Reactive stores
  const activeTabId = $derived($terminalStore.activeTabId);

  function handleTabClick(tabId: string) {
    terminalActions.setActiveTab(tabId);
  }

  function handleCloseTab(tabId: string, event: Event) {
    event.stopPropagation();
    event.preventDefault();
    const wasActive = activeTabId === tabId;
    const remaining = tabs.filter((tab) => tab.id !== tabId);
    terminalActions.closeTab(tabId);
    if (wasActive && remaining.length > 0) {
      terminalActions.setActiveTab(remaining[0].id);
    }
  }

  function handleNewTab() {
    // New tab button clicked
    if (onNewTab) {
      // Calling onNewTab callback
      onNewTab();
    } else {
      // No onNewTab callback provided
    }
  }

  async function loadSystemInfo() {
    try {
      // Loading system info for profile selector
      systemInfo = (await TerminalService.getSystemInfo()) as any;

      if (systemInfo?.terminal_profiles) {
        // Extract available profiles from system info
        const profiles: any[] = [];

        // Add available shells
        if (systemInfo.terminal_profiles.available_shells) {
          Object.entries(systemInfo.terminal_profiles.available_shells).forEach(
            ([name, info]: [string, any]) => {
              profiles.push({
                name,
                command: info.command || name,
                icon: resolveShellIcon(name),
                category: "shell",
              });
            },
          );
        }

        // Add Windows Terminal profiles if available
        if (systemInfo.terminal_profiles.windows_terminal) {
          systemInfo.terminal_profiles.windows_terminal.forEach(
            (profile: any) => {
              profiles.push({
                name: profile.name,
                command: profile.commandline || profile.name,
                icon: resolveShellIcon(profile.name),
                category: "windows_terminal",
              });
            },
          );
        }

        // Remove duplicates based on name
        const uniqueProfiles = profiles.filter(
          (profile, index, self) =>
            index === self.findIndex((p) => p.name === profile.name),
        );

        availableProfiles = uniqueProfiles;

        // Default to a shell with OSC 133 block support. Order matters:
        // pwsh/powershell first — bash on Windows (Git Bash) has no hooks.
        if (availableProfiles.length > 0) {
          const preference = [/pwsh/i, /powershell/i, /zsh/i, /bash/i];
          const preferred = preference
            .map((re) => availableProfiles.find((p) => re.test(p.name)))
            .find(Boolean);
          selectedProfile = (preferred ?? availableProfiles[0]).name;
        }
      } else {
        // No terminal profiles found in system info
        availableProfiles = [];
      }
    } catch (error: any) {
      log.error("Failed to load system info", { error });
      availableProfiles = [];
    }
  }

  function handleProfileChange(value: string) {
    selectedProfile = value;
    // Profile changed
  }

  function createNewTabWithProfile() {
    // Creating new tab with profile

    // Find the profile and extract just the raw command
    const profile = availableProfiles.find((p) => p.name === selectedProfile);

    if (profile) {
      // Using raw shell command
      if (onNewTab) {
        onNewTab(profile.command); // Pass just the raw shell command
      }
    } else {
      // Profile not found
    }
  }

  onMount(() => {
    if (showProfileSelector) {
      loadSystemInfo();
    }
  });

  function getTabStatusColor(tab: any) {
    switch (tab.status) {
      case "active":
        return "border-status-info";
      case "loading":
        return "border-status-warning";
      case "error":
        return "border-status-error";
      default:
        return "border-transparent";
    }
  }

</script>

<div
  class="tab-bar divider-edge-b divider-edge-full flex min-h-[40px] items-center bg-card"
>
  <!-- Tab List -->
  <div class="flex flex-1 overflow-x-auto">
    {#each tabs as tab (tab.id)}
      <div
        onclick={() => handleTabClick(tab.id)}
        class="tab divider-edge-r flex min-w-0 cursor-pointer items-center space-x-2 border-b-2 px-4 py-2 text-sm transition-colors {activeTabId ===
        tab.id
          ? 'bg-accent text-accent-foreground'
          : 'bg-card text-muted-foreground hover:bg-accent/60 hover:text-foreground'} {getTabStatusColor(
          tab,
        )}"
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === "Enter" && handleTabClick(tab.id)}
        title={tab.title}
      >
        <ShellIcon tab={tab} size="sm" />
        <span class="max-w-32 truncate">{tab.title}</span>
        {#if closable && tab.closable !== false && tabs.length > 1}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class="ml-1 h-6 w-6 text-muted-foreground"
            onclick={(e) => handleCloseTab(tab.id, e)}
            aria-label="Close tab"
          >
            <X size={12} />
          </Button>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Profile Selector -->
  {#if showProfileSelector}
    <div class="flex items-center space-x-2 px-2">
      <ShellProfileSelect
        profiles={availableProfiles}
        bind:value={selectedProfile}
        onSelect={handleProfileChange}
        placeholder={availableProfiles.length === 0
          ? "Loading profiles..."
          : "Select shell..."}
        disabled={availableProfiles.length === 0}
      />
    </div>
  {/if}

  <!-- New Tab Button -->
  {#if showNewTabButton && onNewTab}
    <Button
      variant="ghost"
      size="sm"
      onclick={createNewTabWithProfile}
      class="p-2 text-muted-foreground hover:bg-accent hover:text-foreground"
      type="button"
      aria-label="New tab"
      title="New Tab with selected profile (Ctrl+T)"
    >
      <Plus size={16} />
    </Button>
  {/if}
</div>

<style>
  .tab-bar {
    scrollbar-width: thin;
    scrollbar-color: hsl(var(--muted-foreground) / 0.4) hsl(var(--muted));
  }

  .tab-bar::-webkit-scrollbar {
    height: 4px;
  }

  .tab-bar::-webkit-scrollbar-track {
    background: hsl(var(--muted));
  }

  .tab-bar::-webkit-scrollbar-thumb {
    background: hsl(var(--muted-foreground) / 0.4);
    border-radius: 2px;
  }

  .tab-bar::-webkit-scrollbar-thumb:hover {
    background: hsl(var(--muted-foreground) / 0.6);
  }

  .tab {
    white-space: nowrap;
  }
</style>
