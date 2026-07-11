<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { ChevronRight } from "@lucide/svelte";
  import type { ToolCall } from "../types.js";
  import type { CoderSubAgent } from "../types.js";
  import {
    formatCommandCwd,
    formatFailedResult,
    getCompactToolLine,
    getToolResultFailureSummary,
    getToolResultStatus,
    resultLines,
  } from "../utils/toolCallDisplay.js";
  import CoderCommandOutput from "./CoderCommandOutput.svelte";
  import CoderSubAgentInline from "./CoderSubAgentInline.svelte";
  import MarkdownFileContent from "$lib/components/ui/chat-markdown/MarkdownFileContent.svelte";
  import { isMarkdownPath } from "$lib/utils/markdownFile.js";

  interface Props {
    call: ToolCall;
    result?: string | null;
    workspaceRoot?: string;
    threadId?: string;
    subAgents?: CoderSubAgent[];
    coordinatorId?: string;
    onOpenSubAgent?: (childThreadId: string, coordinatorId: string) => void;
    onCancelSubAgent?: (subAgentId: string) => void;
    onCleanupSubAgent?: (subAgentId: string) => void;
    /** Hide outer padding when nested inside activity summary. */
    nested?: boolean;
  }

  let {
    call,
    result = null,
    workspaceRoot = "",
    threadId,
    subAgents = [],
    coordinatorId = "",
    onOpenSubAgent,
    onCancelSubAgent,
    onCleanupSubAgent,
    nested = false,
  }: Props = $props();

  let open = $state(false);

  const args = $derived.by(() => {
    try {
      return JSON.parse(call.function.arguments || "{}") as Record<string, unknown>;
    } catch {
      return {} as Record<string, unknown>;
    }
  });

  const tool = $derived(call.function.name);
  const line = $derived(getCompactToolLine(tool, args));
  const isWrite = $derived(tool === "write_file");
  const writeContent = $derived(isWrite ? String(args.content ?? "") : "");
  const isEdit = $derived(tool === "edit_file");
  const isCommand = $derived(tool === "run_command");
  const command = $derived(isCommand ? String(args.command ?? "") : "");
  const commandCwd = $derived(isCommand ? formatCommandCwd(workspaceRoot) : "");
  const isListDir = $derived(tool === "list_dir");
  const dirEntries = $derived(isListDir && result ? resultLines(result) : []);
  const isSearch = $derived(tool === "search_files");
  const searchHits = $derived(isSearch && result ? resultLines(result) : []);
  const isRead = $derived(tool === "read_file");
  const isSpawnParallel = $derived(tool === "spawn_parallel_tasks");
  const readPath = $derived(isRead ? String(args.path ?? "") : "");
  const isMarkdownRead = $derived(isRead && isMarkdownPath(readPath));
  const isMarkdownWrite = $derived(isWrite && isMarkdownPath(String(args.path ?? "")));
  const status = $derived(getToolResultStatus(tool, result));
  const failed = $derived(status === "failed");
  const failureSummary = $derived(
    failed && result ? getToolResultFailureSummary(tool, result) : "",
  );
  const failedOutput = $derived(
    failed && result ? formatFailedResult(tool, result) : "",
  );
  const hasExpandable = $derived(
    isWrite ||
      isEdit ||
      isCommand ||
      isListDir ||
      isSearch ||
      isRead ||
      isSpawnParallel ||
      tool === "delegate_task" ||
      Object.keys(args).length > 0,
  );
</script>

<div class="{nested ? '' : 'py-0.5'}">
  <Button
    type="button"
    variant="ghost"
    class="h-auto w-full justify-start gap-1.5 px-0 py-0.5 text-left font-normal hover:bg-transparent {failed
      ? 'text-destructive'
      : 'text-muted-foreground hover:text-foreground'}"
    disabled={!hasExpandable && result == null}
    onclick={() => {
      if (hasExpandable || result != null) open = !open;
    }}
  >
    {#if hasExpandable || result != null}
      <ChevronRight
        class="h-3 w-3 shrink-0 transition-transform {open ? 'rotate-90' : ''}"
      />
    {:else}
      <span class="inline-block h-3 w-3 shrink-0" aria-hidden="true"></span>
    {/if}
    <span class="min-w-0 flex-1 truncate text-[11px] leading-snug">
      <span class="text-muted-foreground">{line.verb}</span>
      {#if line.target}
        <span class="font-mono text-foreground/90"> {line.target}</span>
      {/if}
      {#if line.additions != null || line.deletions != null}
        {#if line.additions}
          <span class="text-green-600 dark:text-green-400"> +{line.additions}</span>
        {/if}
        {#if line.deletions}
          <span class="text-red-600 dark:text-red-400"> -{line.deletions}</span>
        {/if}
      {/if}
      {#if status === "pending"}
        <span class="text-muted-foreground/70"> …</span>
      {/if}
      {#if failed && failureSummary}
        <span class="text-destructive"> — {failureSummary}</span>
      {/if}
    </span>
  </Button>

  {#if open}
    <div class="ml-4 mt-1 space-y-2 border-l border-border/60 pl-3 pb-1">
      {#if isWrite}
        {#if isMarkdownWrite}
          <MarkdownFileContent content={writeContent} />
        {:else}
          <pre class="max-h-48 overflow-auto rounded bg-muted/40 p-2 text-[11px]"><code
              >{writeContent}</code
            ></pre>
        {/if}
      {:else if isEdit}
        <div class="space-y-1">
          <pre class="max-h-32 overflow-auto rounded bg-red-500/10 p-2 text-[11px]"><code
              >- {String(args.old_string ?? "")}</code
            ></pre>
          <pre class="max-h-32 overflow-auto rounded bg-green-500/10 p-2 text-[11px]"><code
              >+ {String(args.new_string ?? "")}</code
            ></pre>
        </div>
      {:else if isCommand}
        <div class="space-y-2">
          {#if result !== null && threadId}
            <CoderCommandOutput
              {threadId}
              callId={call.id}
              {command}
              output={failed ? failedOutput : result}
              {failed}
              {workspaceRoot}
            />
          {:else}
            {#if commandCwd}
              <div
                class="truncate font-mono text-[10px] text-muted-foreground"
                title={workspaceRoot}
              >
                {commandCwd}
              </div>
            {/if}
            <div
              class="rounded px-2 py-1 font-mono text-[11px] {failed
                ? 'bg-destructive/10 text-destructive'
                : 'bg-zinc-900 text-zinc-100 dark:bg-zinc-950'}"
            >
              <span class={failed ? 'text-destructive/70' : 'text-zinc-500'}>$</span>
              {command}
            </div>
            {#if result !== null}
              <pre
                class="max-h-48 overflow-auto rounded p-2 text-[11px] {failed
                  ? 'bg-destructive/10 text-destructive'
                  : 'bg-muted/40'}"><code>{failed ? failedOutput : result}</code></pre
              >
            {/if}
          {/if}
        </div>
      {:else if isListDir && result !== null && !failed}
        <ul class="grid max-h-48 grid-cols-2 gap-x-2 gap-y-0.5 overflow-auto text-[11px] sm:grid-cols-3">
          {#each dirEntries as entry}
            <li class="truncate font-mono text-muted-foreground">{entry}</li>
          {/each}
        </ul>
      {:else if isSearch && result !== null && !failed}
        <ul class="max-h-48 space-y-0.5 overflow-auto text-[11px]">
          {#each searchHits as hit}
            <li class="truncate font-mono text-muted-foreground">{hit}</li>
          {/each}
        </ul>
      {:else if failed && result !== null}
        <pre class="max-h-48 overflow-auto rounded bg-destructive/10 p-2 text-[11px] text-destructive"
          ><code>{failedOutput}</code></pre
        >
      {:else if isRead && result !== null}
        {#if isMarkdownRead && !failed}
          <MarkdownFileContent content={result} />
        {:else}
          <pre
            class="max-h-48 overflow-auto rounded p-2 text-[11px] {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-muted/40'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {/if}
      {:else if tool === "delegate_task"}
        <p class="text-[11px] text-muted-foreground">{String(args.goal ?? "")}</p>
        {#if result !== null}
          <pre
            class="max-h-48 overflow-auto rounded p-2 text-[11px] {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-muted/40'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {/if}
      {:else if isSpawnParallel}
        {#if subAgents.length > 0}
          <CoderSubAgentInline
            {subAgents}
            {coordinatorId}
            onOpen={onOpenSubAgent}
            onCancel={onCancelSubAgent}
            onCleanup={onCleanupSubAgent}
            compact={true}
          />
        {:else if result !== null}
          <pre
            class="max-h-48 overflow-auto rounded p-2 text-[11px] {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-muted/40'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {:else}
          <p class="text-[11px] text-muted-foreground">Spawning parallel sub-agents…</p>
        {/if}
      {:else}
        <dl class="space-y-1 text-[11px]">
          {#each Object.entries(args) as [key, value]}
            <div class="flex gap-2">
              <dt class="shrink-0 text-muted-foreground">{key}</dt>
              <dd class="min-w-0 truncate font-mono">{String(value)}</dd>
            </div>
          {/each}
        </dl>
        {#if result !== null}
          <pre
            class="max-h-48 overflow-auto rounded p-2 text-[11px] {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-muted/40'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {/if}
      {/if}
    </div>
  {/if}
</div>
