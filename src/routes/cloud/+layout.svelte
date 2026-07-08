<!-- Cloud Layout - Handles connection guard, namespace selector, and sidebar navigation for all cloud subpages -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import { goto } from "$app/navigation";
  import CloudConnectionGuard from "$lib/domains/cloud/components/CloudConnectionGuard.svelte";
  import CloudNavigation from "$lib/domains/cloud/components/CloudNavigation.svelte";
  import { Card } from "$lib/components/ui/card";
  import CommandPalette from "$lib/domains/k8s-navigation/components/CommandPalette.svelte";
  import ShortcutsHelp from "$lib/domains/k8s-navigation/components/ShortcutsHelp.svelte";
  import {
    useCommandPalette,
    useKeyboardShortcuts,
  } from "$lib/domains/k8s-navigation";
  import { NAVIGATION_SHORTCUTS } from "$lib/domains/k8s-navigation/utils/keyboardConstants";
  import { cloudStore } from "$lib/domains/cloud/stores";
  import { ResourceType } from "$lib/domains/cloud/core/types";
  import type { Command } from "$lib/domains/k8s-navigation";
  import ShellSidebarLayout from "$lib/components/shell/shell-sidebar-layout.svelte";
  import PageContainer from "$lib/components/shell/page-container.svelte";

  let { children }: { children: Snippet<[]> } = $props();

  let showShortcutsHelp = $state(false);

  // Build command palette commands
  const allCommands = $derived.by(() => {
    const commands: Command[] = [];

    // Namespace switching commands
    const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];
    namespaces.forEach((ns: any) => {
      commands.push({
        id: `namespace-${ns.name}`,
        label: `Switch to namespace: ${ns.name}`,
        description: `Switch namespace to ${ns.name}`,
        keywords: ["namespace", "ns", ns.name],
        category: "namespace",
        action: async () => {
          const { setSelectedNamespace } =
            await import("$lib/domains/cloud/stores");
          await setSelectedNamespace(ns.name);
        },
      });
    });

    // Add "All Namespaces" option
    commands.push({
      id: "namespace-all",
      label: "Switch to All Namespaces",
      description: "Show resources from all namespaces",
      keywords: ["namespace", "ns", "all"],
      category: "namespace",
      action: async () => {
        const { setSelectedNamespace } =
          await import("$lib/domains/cloud/stores");
        await setSelectedNamespace("");
      },
    });

    // Resource navigation commands
    commands.push(
      {
        id: "nav-overview",
        label: "Go to Overview",
        description: "Navigate to cluster overview",
        keywords: ["overview", "dashboard"],
        category: "navigation",
        action: () => goto("/cloud/workloads"),
      },
      {
        id: "nav-pods",
        label: "Go to Pods",
        description: "Navigate to pods list",
        keywords: ["pods", "pod"],
        category: "navigation",
        action: () => goto("/cloud/workloads/pods"),
      },
      {
        id: "nav-services",
        label: "Go to Services",
        description: "Navigate to services list",
        keywords: ["services", "service", "svc"],
        category: "navigation",
        action: () => goto("/cloud/workloads/services"),
      },
      {
        id: "nav-deployments",
        label: "Go to Deployments",
        description: "Navigate to deployments list",
        keywords: ["deployments", "deployment", "deploy"],
        category: "navigation",
        action: () => goto("/cloud/workloads/deployments"),
      },
    );

    return commands;
  });

  // Help shortcut
  const helpShortcut = useKeyboardShortcuts(
    [
      {
        key: NAVIGATION_SHORTCUTS.HELP,
        description: "Show keyboard shortcuts help",
        action: () => {
          showShortcutsHelp = true;
        },
      },
    ],
    { enabled: $cloudStore.connection.isConnected },
  );

  function handleKeydown(event: KeyboardEvent) {
    helpShortcut.handleKeydown(event);
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

{#if $cloudStore.connection.isConnected}
  <ShellSidebarLayout>
    {#snippet sidebar()}
      <div class="p-4">
        <Card class="p-3">
          <CloudNavigation />
        </Card>
      </div>
    {/snippet}
    <div class="min-h-0 flex-1 overflow-y-auto">
      <PageContainer variant="full" class="py-4 md:py-6">
        {@render children()}
      </PageContainer>
    </div>
  </ShellSidebarLayout>

  <!-- Navigation Enhancement Components (Additive) -->
  <CommandPalette commands={allCommands} />
  <ShortcutsHelp bind:open={showShortcutsHelp} context="table" />
{:else}
  <CloudConnectionGuard />
{/if}
