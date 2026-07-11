<script lang="ts">
  import { onMount } from "svelte";
  import { Badge } from "$lib/components/ui/badge";
  import CommandBlock from "../CommandBlock.svelte";
  import { commandBlockStore, type CapturedCommand } from "../../stores/commandBlockStore";

  interface Props {
    tabId: string;
    onRerun?: (command: string) => void;
    onExplain?: (block: CapturedCommand) => void;
  }

  let { tabId, onRerun, onExplain }: Props = $props();

  let blocks = $state<CapturedCommand[]>([]);

  onMount(() => {
    commandBlockStore.startShellIntegrationListener();
    return commandBlockStore.subscribeToBlocks(tabId, (next) => {
      blocks = next;
    });
  });

  function clearBlocks() {
    commandBlockStore.clearBlocks(tabId);
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="flex items-center justify-between border-b border-border p-2">
    <div class="flex items-center gap-2">
      <div class="text-sm font-semibold text-foreground">Command Blocks</div>
      <Badge variant="outline" class="text-xs text-muted-foreground">
        {blocks.length} total
      </Badge>
    </div>
    <button
      type="button"
      class="rounded px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={clearBlocks}
      title="Clear command blocks"
    >
      Clear
    </button>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    {#if blocks.length === 0}
      <div class="py-6 text-center text-xs text-muted-foreground">
        No structured commands yet. Run commands in the terminal or use the input bar below.
      </div>
    {:else}
      <div class="space-y-2">
        {#each blocks as block (block.id)}
          <CommandBlock {block} {onRerun} {onExplain} />
        {/each}
      </div>
    {/if}
  </div>
</div>
