<!--
	Folder picker component for selecting directories
	Integrates with Tauri's native file dialog
-->

<script lang="ts">
  import { Button } from "./button";
  import { Input } from "./input";
  import { Label } from "./label";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
    TooltipProvider,
  } from "./tooltip";
  import { invokeClient } from "$lib/utils/invokeClient";
  import { FolderOpen, Info } from "@lucide/svelte";
  import { isTauriEnvironment } from "$lib/utils/tauri";

  interface Props {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    label?: string;
    description?: string;
    onChange?: (path: string) => void;
  }

  let {
    value = $bindable(""),
    placeholder = "/path/to/your/project",
    disabled = false,
    required = false,
    label = "Project Path",
    description = "Select the directory where your project will be located",
    onChange,
  }: Props = $props();

  let isSelecting = $state(false);
  const isTauri = isTauriEnvironment();

  async function handleSelectDirectory() {
    if (disabled || isSelecting || !isTauri) return;

    try {
      isSelecting = true;
      const selectedPath = await invokeClient.post<string | null>(
        "select_directory",
      );

      if (selectedPath) {
        value = selectedPath;
        onChange?.(selectedPath);
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    } finally {
      isSelecting = false;
    }
  }

  function handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onChange?.(target.value);
  }
</script>

<div class="space-y-2">
  {#if label}
    <Label for="folder-picker-input" class="text-sm font-medium">
      {label}
      {#if required}
        <span class="ml-1 text-red-500">*</span>
      {/if}
    </Label>
  {/if}

  <div class="flex space-x-2">
    <Input
      id="folder-picker-input"
      bind:value
      oninput={handleInputChange}
      {placeholder}
      {disabled}
      {required}
      class="flex-1"
    />
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>
          {#snippet child({ props })}
            <Button
              {...props}
              type="button"
              variant="outline"
              size="sm"
              onclick={handleSelectDirectory}
              disabled={disabled || isSelecting || !isTauri}
              class="px-3"
            >
              <FolderOpen class="h-4 w-4" />
              <span class="sr-only">Select Directory</span>
            </Button>
          {/snippet}
        </TooltipTrigger>
        {#if !isTauri}
          <TooltipContent>
            <p class="max-w-xs">
              File browser is only available in the desktop app. This feature
              requires access to the file system which is not available in
              browser mode. Please use the Tauri desktop application to browse
              and select directories.
            </p>
          </TooltipContent>
        {:else}
          <TooltipContent>
            <p>Browse for directory</p>
          </TooltipContent>
        {/if}
      </Tooltip>
    </TooltipProvider>
  </div>

  {#if description}
    <p class="text-xs text-muted-foreground">
      {description}
    </p>
  {/if}
</div>
