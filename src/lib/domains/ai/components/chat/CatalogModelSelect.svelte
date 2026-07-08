<script lang="ts">
  import { cn } from "$lib/utils";
  import * as Popover from "$lib/components/ui/popover";
  import * as Command from "$lib/components/ui/command";
  import { Button } from "$lib/components/ui/button";
  import { Check, ChevronsUpDown } from "@lucide/svelte";
  import ModelSelectOption from "./ModelSelectOption.svelte";
  import type { CatalogModel } from "../../types/index.js";
  import { filterCatalogModels } from "../../utils/catalog.js";

  interface Props {
    models: CatalogModel[];
    value?: string | null;
    onSelect?: (modelId: string) => void;
    placeholder?: string;
    searchPlaceholder?: string;
    disabled?: boolean;
    class?: string;
  }

  let {
    models,
    value = $bindable<string | null>(null),
    onSelect,
    placeholder = "Select model",
    searchPlaceholder = "Search models…",
    disabled = false,
    class: className = "",
  }: Props = $props();

  let open = $state(false);
  let search = $state("");

  const selectedModel = $derived(
    models.find((entry) => entry.id === value) ?? null,
  );

  const filteredModels = $derived(filterCatalogModels(models, search));

  $effect(() => {
    if (!open) search = "";
  });

  function handleSelect(modelId: string) {
    value = modelId;
    onSelect?.(modelId);
    open = false;
    search = "";
  }
</script>

<div class={cn("relative w-full", className)}>
  <Popover.Root bind:open>
    <Popover.Trigger>
      <Button
        variant="outline"
        role="combobox"
        aria-expanded={open}
        {disabled}
        class={cn(
          "h-10 w-full justify-between px-3 font-normal",
          !selectedModel && "text-muted-foreground",
          open && "ring-2 ring-ring ring-offset-2",
        )}
      >
        <span class="min-w-0 flex-1 truncate text-left">
          {#if selectedModel}
            <ModelSelectOption model={selectedModel} compact />
          {:else}
            {placeholder}
          {/if}
        </span>
        <ChevronsUpDown
          class={cn(
            "ms-2 size-4 shrink-0 opacity-50 transition-transform duration-200",
            open && "rotate-180",
          )}
        />
      </Button>
    </Popover.Trigger>

    <Popover.Content
      class="w-[min(100vw-2rem,24rem)] p-0 shadow-md"
      align="start"
      sideOffset={4}
    >
      <Command.Root shouldFilter={false} class="rounded-lg">
        <Command.Input
          placeholder={searchPlaceholder}
          bind:value={search}
          class="h-9"
        />
        <Command.Empty class="py-6 text-center text-sm text-muted-foreground">
          No models found.
        </Command.Empty>
        <Command.List class="max-h-72 overflow-y-auto p-1">
          <Command.Group>
            {#each filteredModels as model (model.id)}
              <Command.Item
                value={model.id}
                onclick={() => handleSelect(model.id)}
                class={cn(
                  "relative flex cursor-pointer select-none items-start rounded-md px-2 py-2.5 outline-none transition-colors",
                  "aria-selected:bg-accent aria-selected:text-accent-foreground",
                  "hover:bg-accent hover:text-accent-foreground",
                  value === model.id && "bg-accent/50",
                )}
              >
                <Check
                  class={cn(
                    "me-2 mt-0.5 size-4 shrink-0 transition-opacity",
                    value === model.id
                      ? "text-primary opacity-100"
                      : "opacity-0",
                  )}
                />
                <div class="min-w-0 flex-1">
                  <ModelSelectOption {model} />
                </div>
              </Command.Item>
            {/each}
          </Command.Group>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Root>
</div>
