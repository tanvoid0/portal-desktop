<script lang="ts">
  import type { CoderSubAgent } from "../types.js";
  import CoderSubAgentCard from "./CoderSubAgentCard.svelte";

  interface Props {
    subAgents: CoderSubAgent[];
    onOpen?: (childThreadId: string, coordinatorId: string) => void;
    onCancel?: (subAgentId: string) => void;
    onCleanup?: (subAgentId: string) => void;
    coordinatorId?: string;
    compact?: boolean;
    showHeader?: boolean;
  }

  let {
    subAgents = [],
    onOpen,
    onCancel,
    onCleanup,
    coordinatorId = "",
    compact = false,
    showHeader = true,
  }: Props = $props();

  const runningCount = $derived(
    subAgents.filter((item) => item.status === "running" || item.status === "pending").length,
  );
</script>

{#if subAgents.length > 0}
  <div
    class="{showHeader
      ? `rounded-lg border border-border/60 bg-muted/20 ${compact ? 'space-y-2 p-2' : 'space-y-2.5 p-3'}`
      : 'space-y-2'}"
  >
    {#if showHeader}
      <div class="flex items-center justify-between gap-2">
        <p class="text-xs font-medium text-muted-foreground">Parallel sub-agents</p>
        {#if runningCount > 0}
          <p class="text-xs text-muted-foreground">
            Waiting for {runningCount} sub-agent{runningCount === 1 ? "" : "s"}…
          </p>
        {/if}
      </div>
    {/if}
    <div class="space-y-2">
      {#each subAgents as subAgent (subAgent.id)}
        <CoderSubAgentCard
          {subAgent}
          {onOpen}
          {onCancel}
          {onCleanup}
          coordinatorId={coordinatorId || subAgent.coordinator_thread_id}
        />
      {/each}
    </div>
  </div>
{/if}
