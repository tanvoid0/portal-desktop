<script lang="ts">
  import {
    terminalStore,
    terminalActions,
  } from "../stores/terminalStore";
  import TabBar from "./TabBar.svelte";
  import { Button } from "$lib/components/ui/button";
  import type { Snippet } from "svelte";
  import type { TerminalTab } from "../stores/terminalStore";

  // Get children snippet from props for Svelte 5
  let {
    children,
    onNewTab = null,
    showNewTabButton = true,
    closable = true,
    className = "",
    tabFilter = (_tab: TerminalTab) => true,
  }: {
    children: Snippet<[]>;
    onNewTab?: ((profileName?: string) => void) | null;
    showNewTabButton?: boolean;
    closable?: boolean;
    className?: string;
    tabFilter?: (tab: TerminalTab) => boolean;
  } = $props();

  // Reactive stores
  let tabs = $derived($terminalStore.tabs.filter(tabFilter));
  let activeTabId = $derived($terminalStore.activeTabId);
  let currentActiveTab = $derived(
    tabs.find((tab) => tab.id === activeTabId) ?? null,
  );

  // Handle new tab creation with profile
  function handleNewTabWithProfile(profileName?: string) {
    if (onNewTab) {
      onNewTab(profileName);
    }
  }

  // Keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+T: New tab
    if (event.ctrlKey && event.key === "t") {
      event.preventDefault();
      if (onNewTab) {
        onNewTab(); // No profile for keyboard shortcut
      }
    }

    // Ctrl+W: Close current tab
    if (event.ctrlKey && event.key === "w") {
      event.preventDefault();
      if (activeTabId && tabs.length > 1) {
        terminalActions.closeTab(activeTabId);
      }
    }

    // Ctrl+Tab: Switch to next tab
    if (event.ctrlKey && event.key === "Tab") {
      event.preventDefault();
      switchToNextTab();
    }

    // Ctrl+Shift+Tab: Switch to previous tab
    if (event.ctrlKey && event.shiftKey && event.key === "Tab") {
      event.preventDefault();
      switchToPreviousTab();
    }

    // Ctrl+1-9: Switch to tab by number
    if (event.ctrlKey && event.key >= "1" && event.key <= "9") {
      event.preventDefault();
      const tabIndex = parseInt(event.key) - 1;
      if (tabIndex < tabs.length) {
        terminalActions.setActiveTab(tabs[tabIndex].id);
      }
    }
  }

  function switchToNextTab() {
    if (tabs.length <= 1) return;

    const currentIndex = tabs.findIndex((tab) => tab.id === activeTabId);
    const nextIndex = (currentIndex + 1) % tabs.length;
    terminalActions.setActiveTab(tabs[nextIndex].id);
  }

  function switchToPreviousTab() {
    if (tabs.length <= 1) return;

    const currentIndex = tabs.findIndex((tab) => tab.id === activeTabId);
    const prevIndex = currentIndex === 0 ? tabs.length - 1 : currentIndex - 1;
    terminalActions.setActiveTab(tabs[prevIndex].id);
  }

  import { onMount, onDestroy } from "svelte";

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="tab-container flex h-full w-full flex-col {className}">
  <!-- Tab Bar -->
  <TabBar
    {tabs}
    onNewTab={handleNewTabWithProfile}
    {showNewTabButton}
    {closable}
  />

  <!-- Tab Content -->
  <div class="min-h-0 flex-1">
    {#if currentActiveTab}
      <div class="h-full w-full">
        {@render children()}
      </div>
    {:else}
      <div class="flex h-full items-center justify-center">
        <div class="text-center">
          <div class="mb-2 text-muted-foreground">No active tab</div>
          {#if onNewTab}
            <Button onclick={() => onNewTab?.()}>Create New Tab</Button>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
