<!--
	Block Library - Block browser and selector
-->
<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import type { Block } from "../types";
  import {
    blockLibraryService,
    blockLibraryStore,
    filteredBlocks,
  } from "../index";

  interface Props {
    onSelect?: (blockId: string) => void;
    onClose?: () => void;
  }

  let { onSelect, onClose }: Props = $props();

  let searchQuery = $state("");
  let selectedCategory = $state<Block["category"] | null>(null);
  let blocks = $state<Block[]>([]);
  let loading = $state(false);

  $effect(() => {
    blockLibraryStore.setSearchQuery(searchQuery);
    blockLibraryStore.setSelectedCategory(selectedCategory);
  });

  onMount(() => {
    (async () => {
      loading = true;
      await blockLibraryStore.loadBlocks();
      loading = false;
    })();

    // Subscribe to filtered blocks
    const unsubscribe = filteredBlocks.subscribe((filtered) => {
      blocks = filtered;
    });

    return () => unsubscribe();
  });
</script>

<Dialog.Root
  open={true}
  onOpenChange={(isOpen) => {
    if (!isOpen) onClose?.();
  }}
>
  <Dialog.Content class="flex max-h-[90vh] max-w-4xl flex-col overflow-hidden">
    <Dialog.Header>
      <div class="flex items-center justify-between">
        <Dialog.Title>Block Library</Dialog.Title>
        <Button variant="ghost" onclick={onClose}>Close</Button>
      </div>
    </Dialog.Header>
    <div class="flex flex-1 flex-col overflow-hidden px-6 pb-6">
      <!-- Search and Filters -->
      <div class="mb-4 space-y-2">
        <Input
          bind:value={searchQuery}
          placeholder="Search blocks..."
          class="w-full"
        />
        <div class="flex flex-wrap gap-2">
          <Button
            variant={selectedCategory === null ? "default" : "outline"}
            size="sm"
            onclick={() => (selectedCategory = null)}
          >
            All
          </Button>
          <Button
            variant={selectedCategory === "build" ? "default" : "outline"}
            size="sm"
            onclick={() => (selectedCategory = "build")}
          >
            Build
          </Button>
          <Button
            variant={selectedCategory === "test" ? "default" : "outline"}
            size="sm"
            onclick={() => (selectedCategory = "test")}
          >
            Test
          </Button>
          <Button
            variant={selectedCategory === "deploy" ? "default" : "outline"}
            size="sm"
            onclick={() => (selectedCategory = "deploy")}
          >
            Deploy
          </Button>
          <Button
            variant={selectedCategory === "utility" ? "default" : "outline"}
            size="sm"
            onclick={() => (selectedCategory = "utility")}
          >
            Utility
          </Button>
        </div>
      </div>

      <!-- Block List -->
      <div class="flex-1 space-y-2 overflow-y-auto">
        {#if loading}
          <p class="py-8 text-center text-muted-foreground">
            Loading blocks...
          </p>
        {:else if blocks.length === 0}
          <p class="py-8 text-center text-muted-foreground">No blocks found</p>
        {:else}
          {#each blocks as block (block.id)}
            <div
              class="cursor-pointer rounded-md border p-4 transition-colors hover:bg-accent"
              onclick={() => onSelect?.(block.id)}
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <h3 class="font-medium">{block.name}</h3>
                  <p class="mt-1 text-sm text-muted-foreground">
                    {block.description}
                  </p>
                  <div class="mt-2 flex gap-2">
                    <span
                      class="rounded bg-primary/10 px-2 py-1 text-xs text-primary"
                    >
                      {block.category}
                    </span>
                    {#each block.tags.slice(0, 3) as tag}
                      <span
                        class="rounded bg-muted px-2 py-1 text-xs text-muted-foreground"
                      >
                        {tag}
                      </span>
                    {/each}
                  </div>
                </div>
                <Button size="sm" onclick={() => onSelect?.(block.id)}>
                  Add
                </Button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
