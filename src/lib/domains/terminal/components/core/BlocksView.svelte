<!--
  BlocksView — Warp-style primary surface: chronological command blocks
  (oldest → newest), auto-scrolled to the latest. Pairs with the raw xterm
  pane via the view toggle in TerminalSession.
-->
<script lang="ts">
  import { onMount, tick } from "svelte";
  import { Terminal as TerminalIcon } from "@lucide/svelte";
  import CommandBlock from "../CommandBlock.svelte";
  import {
    commandBlockStore,
    type CapturedCommand,
  } from "../../stores/commandBlockStore";

  interface Props {
    tabId: string;
    onRerun?: (command: string) => void;
    onExplain?: (block: CapturedCommand) => void;
  }

  let { tabId, onRerun, onExplain }: Props = $props();

  let blocks = $state<CapturedCommand[]>([]);
  let scrollEl = $state<HTMLDivElement | null>(null);
  let stickToBottom = true;

  // Store keeps newest-first; render oldest-first like a terminal.
  const ordered = $derived([...blocks].reverse());

  onMount(() => {
    commandBlockStore.startShellIntegrationListener();
    return commandBlockStore.subscribeToBlocks(tabId, async (next) => {
      blocks = next;
      if (stickToBottom) {
        await tick();
        scrollEl?.scrollTo({ top: scrollEl.scrollHeight });
      }
    });
  });

  function handleScroll() {
    if (!scrollEl) return;
    stickToBottom =
      scrollEl.scrollHeight - scrollEl.scrollTop - scrollEl.clientHeight < 40;
  }
</script>

<div
  bind:this={scrollEl}
  onscroll={handleScroll}
  class="h-full min-h-0 overflow-y-auto bg-background p-3"
>
  {#if ordered.length === 0}
    <div class="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
      <TerminalIcon class="h-8 w-8" />
      <p class="text-sm">Run a command below — each one becomes a block.</p>
      <p class="text-xs">
        Use <kbd class="rounded bg-muted px-1 text-foreground">Ctrl+Space</kbd> for AI, or switch
        to Terminal view for interactive apps (vim, htop…).
      </p>
    </div>
  {:else}
    <div class="mx-auto flex max-w-4xl flex-col gap-2">
      {#each ordered as block (block.id)}
        <CommandBlock {block} {onRerun} {onExplain} />
      {/each}
    </div>
  {/if}
</div>
