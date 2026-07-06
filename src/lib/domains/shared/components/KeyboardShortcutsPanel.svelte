<!-- Generic Keyboard Shortcuts Panel Component - Reusable anywhere -->
<!-- Can be customized per page/domain to show relevant shortcuts -->

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Keyboard, X } from "@lucide/svelte";
  import KeyboardShortcutHint from "$lib/domains/k8s-navigation/components/KeyboardShortcutHint.svelte";

  export interface ShortcutGroup {
    title: string;
    shortcuts: Array<{ key: string; description: string; hint?: string }>;
  }

  interface Props {
    shortcuts:
      | Array<{ key: string; description: string; category?: string }>
      | ShortcutGroup[];
    title?: string;
    variant?: "panel" | "modal" | "inline" | "tooltip";
    showTitle?: boolean;
    showCategoryHeaders?: boolean;
    collapsible?: boolean;
    maxHeight?: string;
    position?: "top" | "bottom" | "left" | "right";
  }

  let {
    shortcuts,
    title = "Keyboard Shortcuts",
    variant = "panel",
    showTitle = true,
    showCategoryHeaders = true,
    collapsible = false,
    maxHeight = "400px",
    position = "bottom",
  }: Props = $props();

  let isExpanded = $state(!collapsible);
  let isOpen = $state(false);

  // Normalize shortcuts to groups
  const shortcutGroups = $derived(() => {
    // If already in group format
    if (
      shortcuts.length > 0 &&
      "title" in shortcuts[0] &&
      "shortcuts" in shortcuts[0]
    ) {
      return shortcuts as ShortcutGroup[];
    }

    // Convert flat array to groups by category
    const flatShortcuts = shortcuts as Array<{
      key: string;
      description: string;
      category?: string;
    }>;
    const groups = new Map<
      string,
      Array<{ key: string; description: string }>
    >();

    flatShortcuts.forEach((shortcut) => {
      const category = shortcut.category || "Other";
      if (!groups.has(category)) {
        groups.set(category, []);
      }
      groups
        .get(category)!
        .push({ key: shortcut.key, description: shortcut.description });
    });

    return Array.from(groups.entries()).map(([title, shortcuts]) => ({
      title,
      shortcuts,
    }));
  });

  function toggle() {
    if (collapsible) {
      isExpanded = !isExpanded;
    }
  }

  function openModal() {
    isOpen = true;
  }

  function closeModal() {
    isOpen = false;
  }
</script>

{#if variant === "inline"}
  <div class="space-y-2">
    {#if showTitle}
      <h3 class="flex items-center gap-2 text-sm font-semibold">
        <Keyboard class="h-4 w-4" />
        {title}
      </h3>
    {/if}
    <div class="space-y-3">
      {#each shortcutGroups() as group}
        <div>
          {#if showCategoryHeaders}
            <h4
              class="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground"
            >
              {group.title}
            </h4>
          {/if}
          <div class="space-y-1.5">
            {#each group.shortcuts as shortcut}
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground">{shortcut.description}</span
                >
                <KeyboardShortcutHint shortcut={shortcut.key} />
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
{:else if variant === "panel"}
  <Card>
    <CardHeader class="pb-3">
      <div class="flex items-center justify-between">
        <CardTitle class="flex items-center gap-2 text-base">
          <Keyboard class="h-4 w-4" />
          {title}
        </CardTitle>
        {#if collapsible}
          <Button variant="ghost" size="sm" onclick={toggle}>
            {#if isExpanded}
              <X class="h-4 w-4" />
            {:else}
              <Keyboard class="h-4 w-4" />
            {/if}
          </Button>
        {/if}
      </div>
    </CardHeader>
    {#if isExpanded}
      <CardContent>
        <div
          class="space-y-4"
          style="max-height: {maxHeight}; overflow-y: auto;"
        >
          {#each shortcutGroups() as group}
            <div>
              {#if showCategoryHeaders}
                <h4
                  class="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >
                  {group.title}
                </h4>
              {/if}
              <div class="space-y-1.5">
                {#each group.shortcuts as shortcut}
                  <div class="flex items-center justify-between py-1 text-sm">
                    <span class="text-muted-foreground"
                      >{shortcut.description}</span
                    >
                    <KeyboardShortcutHint shortcut={shortcut.key} />
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </CardContent>
    {/if}
  </Card>
{:else if variant === "modal"}
  <!-- Modal variant -->
  <Button variant="outline" size="sm" onclick={openModal}>
    <Keyboard class="mr-2 h-4 w-4" />
    Shortcuts
  </Button>

  <Dialog.Root bind:open={isOpen}>
    <Dialog.Content class="max-h-[80vh] max-w-2xl overflow-hidden">
      <Dialog.Header class="pb-3">
        <Dialog.Title class="flex items-center gap-2 text-lg">
          <Keyboard class="h-5 w-5" />
          {title}
        </Dialog.Title>
      </Dialog.Header>
      <div
        class="space-y-6 px-6 pb-6"
        style="max-height: calc(80vh - 120px); overflow-y: auto;"
      >
        {#each shortcutGroups() as group}
          <div>
            {#if showCategoryHeaders}
              <h4 class="mb-3 text-sm font-semibold">{group.title}</h4>
            {/if}
            <div class="grid grid-cols-2 gap-x-4 gap-y-2">
              {#each group.shortcuts as shortcut}
                <div class="flex items-center justify-between text-sm">
                  <span class="text-muted-foreground"
                    >{shortcut.description}</span
                  >
                  <KeyboardShortcutHint shortcut={shortcut.key} />
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </Dialog.Content>
  </Dialog.Root>
{:else if variant === "tooltip"}
  <!-- Tooltip variant - minimal inline display -->
  <div class="inline-flex items-center gap-1 text-xs text-muted-foreground">
    <Keyboard class="h-3 w-3" />
    <span>Press ? for shortcuts</span>
  </div>
{/if}
