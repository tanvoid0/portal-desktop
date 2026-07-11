<!--
	Command Input Component
	Enhanced input field with AI suggestions and command history
	Supports inline AI mode with /ai prefix or Ctrl+Space toggle
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { Sparkles } from "@lucide/svelte";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { commandHistoryStore } from "../../stores/commandHistoryStore";
  import { cn } from "$lib/utils";

  interface Props {
    tabId?: string;
    onSubmit: (command: string, isAIMode?: boolean) => void;
    onIntercept?: (command: string) => boolean;
    onStop?: () => void;
    disabled?: boolean;
    placeholder?: string;
  }

  let {
    tabId = "global",
    onSubmit,
    onIntercept,
    onStop,
    disabled = false,
    placeholder = "Enter command or /ai for AI mode...",
  }: Props = $props();

  let inputValue = $state("");
  let historyIndex = $state(-1);
  let suggestions = $state<string[]>([]);
  let showSuggestions = $state(false);
  let inputRef = $state<HTMLInputElement | null>(null);
  let isAIMode = $state(false);

  // Detect AI mode from input
  $effect(() => {
    isAIMode = inputValue.trim().startsWith("/ai ");
  });

  // Get command history for current tab
  const history = $derived.by(() => {
    return commandHistoryStore.getTabHistory(tabId) || [];
  });

  // Filter suggestions based on input
  $effect(() => {
    if (inputValue.trim()) {
      const filtered = history
        .filter((entry) =>
          entry.command.toLowerCase().includes(inputValue.toLowerCase()),
        )
        .slice(0, 5)
        .map((entry) => entry.command);
      suggestions = filtered;
      showSuggestions = filtered.length > 0;
    } else {
      suggestions = [];
      showSuggestions = false;
    }
  });

  function handleSubmit() {
    const command = inputValue.trim();
    if (!command || disabled) return;

    // Extract actual command and AI mode
    let actualCommand = command;
    let aiMode = false;

    if (command.startsWith("/ai ")) {
      actualCommand = command.substring(4).trim();
      aiMode = true;
    }

    // Check if command should be intercepted
    if (onIntercept && onIntercept(actualCommand)) {
      return; // Intercepted, don't submit
    }

    onSubmit(actualCommand, aiMode);
    inputValue = "";
    historyIndex = -1;
    showSuggestions = false;
  }

  function toggleAIMode() {
    if (isAIMode) {
      // Remove /ai prefix
      inputValue = inputValue.substring(4).trim();
    } else {
      // Add /ai prefix
      inputValue = "/ai " + inputValue.trim();
    }
    // Focus input
    if (inputRef) {
      inputRef.focus();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Handle Ctrl+C to stop running command
    if (e.key === "c" && e.ctrlKey) {
      e.preventDefault();
      if (onStop) {
        onStop();
      }
      return;
    }

    // Handle Ctrl+Space to toggle AI mode
    if (e.key === " " && e.ctrlKey) {
      e.preventDefault();
      toggleAIMode();
      return;
    }

    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (historyIndex < history.length - 1) {
        historyIndex++;
        inputValue = history[history.length - 1 - historyIndex].command;
        showSuggestions = false;
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIndex > 0) {
        historyIndex--;
        inputValue = history[history.length - 1 - historyIndex].command;
      } else if (historyIndex === 0) {
        historyIndex = -1;
        inputValue = "";
      }
    } else if (e.key === "Escape") {
      showSuggestions = false;
      if (inputRef) {
        inputRef.blur();
      }
    } else if (e.key === "Tab" && suggestions.length > 0 && showSuggestions) {
      e.preventDefault();
      inputValue = suggestions[0];
      showSuggestions = false;
    }
  }

  function selectSuggestion(suggestion: string) {
    inputValue = suggestion;
    showSuggestions = false;
    if (inputRef) {
      inputRef.focus();
    }
  }

  function focusInput() {
    if (inputRef) {
      inputRef.focus();
    }
  }
</script>

<div class="relative">
  <div
    class={cn(
      "flex items-center gap-2 rounded-lg border bg-background px-3 py-1.5 transition-colors",
      isAIMode
        ? "border-primary/60 focus-within:border-primary"
        : "border-border focus-within:border-ring",
    )}
  >
    <!-- Mode indicator -->
    <div class="flex-shrink-0">
      {#if isAIMode}
        <Sparkles class="h-4 w-4 text-primary" />
      {:else}
        <span class="font-mono text-sm font-bold text-status-success">❯</span>
      {/if}
    </div>

    <!-- Input field -->
    <div class="relative min-w-0 flex-1">
      <Input
        bind:ref={inputRef}
        bind:value={inputValue}
        onkeydown={handleKeydown}
        {disabled}
        {placeholder}
        class="h-8 border-0 bg-transparent font-mono text-sm text-foreground shadow-none placeholder:text-muted-foreground focus-visible:ring-0 focus-visible:ring-offset-0"
      />

      <!-- Suggestions Dropdown (opens upward, above the input) -->
      {#if showSuggestions && suggestions.length > 0}
        <div
          class="suggestions-dropdown absolute bottom-full z-10 mb-1 max-h-48 w-full overflow-y-auto rounded-lg border border-border bg-popover shadow-lg"
        >
          {#each suggestions as suggestion}
            <Button
              type="button"
              variant="ghost"
              onclick={() => selectSuggestion(suggestion)}
              class="h-auto w-full justify-start rounded-none px-3 py-1.5 font-mono text-xs text-popover-foreground"
            >
              {suggestion}
            </Button>
          {/each}
        </div>
      {/if}
    </div>

    {#if isAIMode}
      <Badge
        variant="secondary"
        class="pointer-events-none flex-shrink-0 border-primary/30 bg-primary/20 text-[10px] text-primary"
      >
        AI
      </Badge>
    {/if}

    <!-- AI Mode Toggle -->
    <Button
      variant="ghost"
      size="sm"
      onclick={toggleAIMode}
      {disabled}
      class={cn(
        "h-6 w-6 flex-shrink-0 p-0",
        isAIMode ? "text-primary" : "text-muted-foreground hover:text-foreground",
      )}
      title="Toggle AI Mode (Ctrl+Space)"
    >
      <Sparkles class="h-3.5 w-3.5" />
    </Button>

    <!-- Submit -->
    <Button
      onclick={handleSubmit}
      disabled={disabled || !inputValue.trim()}
      variant="ghost"
      size="sm"
      class="h-6 flex-shrink-0 px-2 text-xs text-muted-foreground hover:text-foreground"
    >
      Run ⏎
    </Button>
  </div>
</div>

<style>
  /* Suggestions scrollbar */
  :global(.suggestions-dropdown) {
    scrollbar-width: thin;
    scrollbar-color: hsl(var(--muted-foreground) / 0.4) hsl(var(--muted));
  }

  :global(.suggestions-dropdown)::-webkit-scrollbar {
    width: 6px;
  }

  :global(.suggestions-dropdown)::-webkit-scrollbar-track {
    background: hsl(var(--muted));
  }

  :global(.suggestions-dropdown)::-webkit-scrollbar-thumb {
    background: hsl(var(--muted-foreground) / 0.4);
    border-radius: 3px;
  }
</style>
