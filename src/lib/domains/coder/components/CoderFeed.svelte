<script lang="ts">
  import type { ChatMessage, CoderSubAgent } from "../types.js";
  import { buildFeedBlocks } from "../utils/feedBlocks.js";
  import CoderUserBlock from "./CoderUserBlock.svelte";
  import CoderAssistantBlock from "./CoderAssistantBlock.svelte";
  import CoderThoughtBlock from "./CoderThoughtBlock.svelte";
  import CoderActivitySummary from "./CoderActivitySummary.svelte";
  import ToolCallCard from "./ToolCallCard.svelte";
  import CoderSubAgentInline from "./CoderSubAgentInline.svelte";
  import ApprovalBanner from "./ApprovalBanner.svelte";
  import type { PendingApproval } from "../types.js";
  import { stripLeakedToolSyntax } from "../utils/leakedToolCall.js";
  import { Button } from "$lib/components/ui/button";
  import { RotateCcw } from "@lucide/svelte";

  interface Props {
    messages: ChatMessage[];
    streamingText?: string;
    /** Seconds waited on the current LLM step with no output yet (0 = not waiting). */
    waitingSeconds?: number;
    running?: boolean;
    pending?: PendingApproval | null;
    error?: string | null;
    showRetry?: boolean;
    showSubAgentInline?: boolean;
    subAgents?: CoderSubAgent[];
    coordinatorId?: string;
    workspaceRoot?: string;
    threadId?: string;
    canEdit?: boolean;
    onEditMessage?: (messageIndex: number, content: string) => void;
    onRetry?: () => void;
    onOpenSubAgent?: (childId: string, coordId: string) => void;
    onCancelSubAgent?: (id: string) => void;
    onCleanupSubAgent?: (id: string) => void;
    onDecision?: (
      approve: boolean,
      remember: boolean,
      editedPattern?: string,
    ) => void;
  }

  let {
    messages,
    streamingText = "",
    waitingSeconds = 0,
    running = false,
    pending = null,
    error = null,
    showRetry = false,
    showSubAgentInline = false,
    subAgents = [],
    coordinatorId = "",
    workspaceRoot = "",
    threadId,
    canEdit = false,
    onEditMessage,
    onRetry,
    onOpenSubAgent,
    onCancelSubAgent,
    onCleanupSubAgent,
    onDecision,
  }: Props = $props();

  const resultsById = $derived.by(() => {
    const map = new Map<string, string>();
    for (const m of messages) {
      if (m.role === "tool" && m.tool_call_id) {
        map.set(m.tool_call_id, m.content ?? "");
      }
    }
    return map;
  });

  const blocks = $derived(buildFeedBlocks(messages, resultsById));
  const displayStreamingText = $derived(stripLeakedToolSyntax(streamingText));

  function blockKey(block: (typeof blocks)[number]): string {
    if (block.kind === "tool") return block.call.id;
    if (block.kind === "activity") {
      return `activity-${block.tools.map((t) => t.call.id).join("-")}`;
    }
    return `${block.kind}-${block.messageIndex}`;
  }
</script>

<div class="space-y-3">
  {#each blocks as block (blockKey(block))}
    {#if block.kind === "user"}
      <CoderUserBlock
        message={block.message}
        messageIndex={block.messageIndex}
        {canEdit}
        onEdit={onEditMessage}
      />
    {:else if block.kind === "thought"}
      <CoderThoughtBlock
        message={block.message}
        durationMs={block.durationMs}
      />
    {:else if block.kind === "assistant"}
      <CoderAssistantBlock
        message={block.message}
        responseLatencyMs={block.responseLatencyMs}
      />
    {:else if block.kind === "activity"}
      <CoderActivitySummary
        parts={block.parts}
        isRunning={block.isRunning}
        hasFailed={block.hasFailed}
      >
        {#snippet children()}
          {#each block.tools as tool (tool.call.id)}
            <ToolCallCard
              call={tool.call}
              result={tool.result}
              {workspaceRoot}
              {threadId}
              {subAgents}
              {coordinatorId}
              onOpenSubAgent={onOpenSubAgent}
              onCancelSubAgent={onCancelSubAgent}
              onCleanupSubAgent={onCleanupSubAgent}
              nested={true}
            />
          {/each}
        {/snippet}
      </CoderActivitySummary>
    {:else if block.kind === "tool"}
      <ToolCallCard
        call={block.call}
        result={block.result}
        {workspaceRoot}
        {threadId}
        {subAgents}
        {coordinatorId}
        onOpenSubAgent={onOpenSubAgent}
        onCancelSubAgent={onCancelSubAgent}
        onCleanupSubAgent={onCleanupSubAgent}
      />
    {/if}
  {/each}

  {#if displayStreamingText}
    <CoderAssistantBlock
      message={{ role: "assistant", content: displayStreamingText }}
      isStreaming={true}
    />
  {/if}

  {#if running && !pending && !displayStreamingText}
    <CoderAssistantBlock
      message={{ role: "assistant", content: "" }}
      showLoader={true}
      {waitingSeconds}
    />
  {/if}

  {#if showSubAgentInline}
    <CoderSubAgentInline
      {subAgents}
      {coordinatorId}
      onOpen={onOpenSubAgent}
      onCancel={onCancelSubAgent}
      onCleanup={onCleanupSubAgent}
    />
  {/if}

  {#if pending && onDecision}
    <ApprovalBanner
      {pending}
      busy={running}
      onDecision={onDecision}
    />
  {/if}

  {#if showRetry && !running}
    {#if error}
      <div
        class="flex items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-[11px] text-destructive"
      >
        <span class="flex-1">{error}</span>
        {#if onRetry}
          <Button
            size="sm"
            variant="outline"
            class="h-7 shrink-0 gap-1 border-destructive/40 text-destructive hover:bg-destructive/10"
            onclick={onRetry}
          >
            <RotateCcw class="h-3.5 w-3.5" />
            Retry
          </Button>
        {/if}
      </div>
    {:else}
      <div
        class="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border bg-muted/30 px-4 py-4 text-center"
      >
        <p class="text-[11px] text-muted-foreground">
          The agent didn't reply. Retry to run the turn again.
        </p>
        {#if onRetry}
          <Button
            size="sm"
            variant="outline"
            class="h-7 gap-1"
            onclick={onRetry}
          >
            <RotateCcw class="h-3.5 w-3.5" />
            Retry
          </Button>
        {/if}
      </div>
    {/if}
  {:else if error}
    <div
      class="flex items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-[11px] text-destructive"
    >
      <span class="flex-1">{error}</span>
    </div>
  {/if}
</div>
