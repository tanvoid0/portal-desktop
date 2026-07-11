<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { ChevronDown, ChevronRight, GitBranchPlus, Trash2 } from "@lucide/svelte";
  import type { CoderSubAgent } from "../types.js";
  import CoderSubAgentInline from "./CoderSubAgentInline.svelte";

  interface Props {
    subAgents: CoderSubAgent[];
    coordinatorId?: string;
    onOpen?: (childThreadId: string, coordinatorId: string) => void;
    onCancel?: (subAgentId: string) => void;
    onCleanupOne?: (subAgentId: string) => void;
    onCleanupAll?: () => void;
  }

  let {
    subAgents = [],
    coordinatorId = "",
    onOpen,
    onCancel,
    onCleanupOne,
    onCleanupAll,
  }: Props = $props();

  let expanded = $state(true);
  const runningCount = $derived(
    subAgents.filter((item) => item.status === "running" || item.status === "pending").length,
  );
</script>

<div class="divider-edge-b divider-edge-full bg-muted/20 px-4 py-2">
  <div class="flex items-center justify-between gap-2">
    <Button
      type="button"
      variant="ghost"
      class="h-auto min-w-0 flex-1 justify-start gap-2 px-0 text-left"
      onclick={() => (expanded = !expanded)}
    >
      {#if expanded}
        <ChevronDown class="h-4 w-4 text-muted-foreground" />
      {:else}
        <ChevronRight class="h-4 w-4 text-muted-foreground" />
      {/if}
      <GitBranchPlus class="h-4 w-4 text-primary" />
      <div class="min-w-0">
        <p class="text-sm font-medium">Multitask</p>
        <p class="text-xs text-muted-foreground">
          {runningCount} working, {subAgents.length} total
        </p>
      </div>
    </Button>
    {#if subAgents.length > 0}
      <Button
        size="sm"
        variant="ghost"
        class="h-7 gap-1 text-xs"
        title="Clean up completed worktrees"
        onclick={() => onCleanupAll?.()}
      >
        <Trash2 class="h-3.5 w-3.5" />
        Cleanup
      </Button>
    {/if}
  </div>

  {#if expanded && subAgents.length > 0}
    <div class="mt-2">
    <CoderSubAgentInline
      {subAgents}
      {coordinatorId}
      {onOpen}
      onCancel={onCancel}
      onCleanup={onCleanupOne}
      compact={true}
      showHeader={false}
    />
    </div>
  {/if}
</div>
