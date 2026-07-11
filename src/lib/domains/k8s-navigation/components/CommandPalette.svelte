<!-- Command Palette - Overlay for quick commands -->
<script lang="ts">
  import { useCommandPalette } from "../hooks/useCommandPalette";
  import type { Command } from "../types";
  import { Dialog, DialogContent } from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { onMount } from "svelte";
  import { get } from "svelte/store";

  interface Props {
    commands: Command[];
    onClose?: () => void;
  }

  let { commands, onClose }: Props = $props();

  const palette = useCommandPalette({ commands });

  // Extract stores for easier access
  const isOpenStore = palette.isOpen;
  const queryStore = palette.query;
  const selectedIndexStore = palette.selectedIndex;
  const filteredCommandsStore = palette.filteredCommands;

  let inputElement: HTMLInputElement | null = null;

  // Handle keyboard events
  function handleKeydown(event: KeyboardEvent) {
    if (palette.handleKeydown(event)) {
      return;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });

  $effect(() => {
    if ($isOpenStore && inputElement) {
      setTimeout(() => {
        inputElement?.focus();
      }, 0);
    }
  });

  function handleClose() {
    palette.close();
    onClose?.();
  }
</script>

<Dialog
  open={$isOpenStore}
  onOpenChange={(open) => {
    if (!open) {
      handleClose();
    } else {
      isOpenStore.set(open);
    }
  }}
>
  <DialogContent class="p-0 sm:max-w-[600px]">
    <div class="flex flex-col">
      <!-- Search Input -->
      <div class="divider-edge-b divider-edge-full p-4">
        <Input
          bind:ref={inputElement}
          type="text"
          placeholder="Type a command or search..."
          bind:value={$queryStore}
        />
      </div>

      <!-- Command List -->
      <ScrollArea class="max-h-[400px]">
        <div class="p-2">
          {#if $filteredCommandsStore.length === 0}
            <div class="p-4 text-center text-sm text-muted-foreground">
              No commands found
            </div>
          {:else}
            <div class="space-y-1">
              {#each $filteredCommandsStore as command, index}
                {@const isSelected = index === $selectedIndexStore}
                <Button
                  type="button"
                  variant="ghost"
                  class="h-auto w-full justify-start rounded-md px-3 py-2 text-left text-sm transition-colors {isSelected
                    ? 'bg-accent text-accent-foreground'
                    : 'hover:bg-muted'}"
                  onclick={async () => {
                    const currentCommands = $filteredCommandsStore;
                    const commandIndex = currentCommands.indexOf(command);
                    if (commandIndex >= 0) {
                      await command.action();
                      handleClose();
                    }
                  }}
                >
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="font-medium">{command.label}</div>
                      {#if command.description}
                        <div class="mt-0.5 text-xs text-muted-foreground">
                          {command.description}
                        </div>
                      {/if}
                    </div>
                    {#if command.category}
                      <span class="ml-2 text-xs text-muted-foreground">
                        {command.category}
                      </span>
                    {/if}
                  </div>
                </Button>
              {/each}
            </div>
          {/if}
        </div>
      </ScrollArea>

      <!-- Footer -->
      <div
        class="divider-edge-t divider-edge-full flex items-center justify-between p-2 text-xs text-muted-foreground"
      >
        <div class="flex items-center gap-4">
          <span>↑↓ Navigate</span>
          <span>Enter Select</span>
          <span>Esc Close</span>
        </div>
      </div>
    </div>
  </DialogContent>
</Dialog>
