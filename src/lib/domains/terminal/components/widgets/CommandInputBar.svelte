<script lang="ts">
  import { Blocks, SquareTerminal } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import CommandInput from "../ai/CommandInput.svelte";
  import type { SessionView } from "../core/TerminalSession.svelte";

  interface Props {
    tabId: string;
    onSubmit: (command: string, isAIMode?: boolean) => void;
    onStop?: () => void;
    /** Current session view; omit to hide the toggle. */
    view?: SessionView;
    onViewChange?: (view: SessionView) => void;
  }

  let { tabId, onSubmit, onStop, view, onViewChange }: Props = $props();
</script>

<div class="flex items-center gap-2 border-t border-border bg-card p-2">
  {#if view && onViewChange}
    <div class="flex flex-shrink-0 overflow-hidden rounded-md border border-border">
      <Button
        variant={view === "blocks" ? "secondary" : "ghost"}
        size="sm"
        class="h-8 rounded-none px-2"
        title="Blocks view (Warp-style)"
        onclick={() => onViewChange("blocks")}
      >
        <Blocks class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant={view === "terminal" ? "secondary" : "ghost"}
        size="sm"
        class="h-8 rounded-none px-2"
        title="Terminal view (interactive apps)"
        onclick={() => onViewChange("terminal")}
      >
        <SquareTerminal class="h-3.5 w-3.5" />
      </Button>
    </div>
  {/if}
  <div class="min-w-0 flex-1">
    <CommandInput
      {tabId}
      {onSubmit}
      {onStop}
      placeholder="Enter command — Ctrl+Space for AI..."
    />
  </div>
</div>
