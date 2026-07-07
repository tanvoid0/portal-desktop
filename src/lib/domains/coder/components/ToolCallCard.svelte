<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import {
    Bot,
    ChevronRight,
    FileText,
    FolderOpen,
    Pencil,
    PenLine,
    Search,
    Terminal,
  } from "@lucide/svelte";
  import type { Component } from "svelte";
  import type { ToolCall } from "../types.js";
  import {
    formatCommandCwd,
    formatFailedResult,
    getToolCallDisplay,
    getToolResultFailureSummary,
    getToolResultStatus,
    resultLines,
  } from "../utils/toolCallDisplay.js";

  interface Props {
    call: ToolCall;
    /** The matching `tool` message content, if the call already ran. */
    result?: string | null;
    /** Workspace root where local tools execute. */
    workspaceRoot?: string;
  }

  let { call, result = null, workspaceRoot = "" }: Props = $props();

  let open = $state(false);

  const args = $derived.by(() => {
    try {
      return JSON.parse(call.function.arguments || "{}") as Record<string, unknown>;
    } catch {
      return {} as Record<string, unknown>;
    }
  });

  const tool = $derived(call.function.name);
  const display = $derived(getToolCallDisplay(tool, args));
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
  const status = $derived(getToolResultStatus(tool, result));
  const failed = $derived(status === "failed");
  const failureSummary = $derived(
    failed && result ? getToolResultFailureSummary(tool, result) : "",
  );
  const failedOutput = $derived(
    failed && result ? formatFailedResult(tool, result) : "",
  );

  const Icon = $derived.by((): Component => {
    switch (tool) {
      case "read_file":
        return FileText;
      case "write_file":
        return PenLine;
      case "edit_file":
        return Pencil;
      case "list_dir":
        return FolderOpen;
      case "search_files":
        return Search;
      case "run_command":
        return Terminal;
      case "delegate_task":
        return Bot;
      default:
        return Terminal;
    }
  });
</script>

<div
  class="rounded-md border text-sm {failed
    ? 'border-destructive/40 bg-destructive/5'
    : 'border-border bg-muted/40'}"
>
  <button
    type="button"
    class="flex w-full items-center gap-2 px-3 py-2 text-left"
    onclick={() => (open = !open)}
  >
    <ChevronRight
      class="h-3.5 w-3.5 shrink-0 transition-transform {open ? 'rotate-90' : ''}"
    />
    <Icon
      class="h-3.5 w-3.5 shrink-0 {failed
        ? 'text-destructive'
        : 'text-muted-foreground'}"
    />
    <span class="min-w-0 flex-1">
      <span class="text-xs font-medium">{display.label}</span>
      {#if display.detail}
        <span class="mt-0.5 block truncate font-mono text-[11px] text-muted-foreground">
          {display.detail}
        </span>
      {/if}
      {#if commandCwd}
        <span
          class="mt-0.5 block truncate font-mono text-[11px] text-muted-foreground/80"
          title={workspaceRoot}
        >
          {commandCwd}
        </span>
      {/if}
      {#if failed && failureSummary}
        <span class="mt-0.5 block truncate text-[11px] text-destructive">
          {failureSummary}
        </span>
      {/if}
    </span>
    {#if status === "failed"}
      <Badge variant="destructive" class="shrink-0 text-[10px]">failed</Badge>
    {:else if status === "success"}
      <Badge variant="secondary" class="shrink-0 text-[10px]">done</Badge>
    {:else}
      <Badge variant="outline" class="shrink-0 text-[10px]">pending</Badge>
    {/if}
  </button>

  {#if open}
    <div class="space-y-2 border-t border-border px-3 py-2">
      {#if isWrite}
        <div>
          <pre class="max-h-64 overflow-auto rounded bg-background p-2 text-xs"><code
              >{writeContent}</code
            ></pre>
        </div>
      {:else if isEdit}
        <div class="space-y-1">
          <pre class="max-h-40 overflow-auto rounded bg-red-500/10 p-2 text-xs"><code
              >- {String(args.old_string ?? "")}</code
            ></pre>
          <pre class="max-h-40 overflow-auto rounded bg-green-500/10 p-2 text-xs"><code
              >+ {String(args.new_string ?? "")}</code
            ></pre>
        </div>
      {:else if isCommand}
        <div class="space-y-2">
          {#if workspaceRoot}
            <div
              class="truncate font-mono text-[11px] text-muted-foreground"
              title={workspaceRoot}
            >
              {workspaceRoot}
            </div>
          {/if}
          <div
            class="rounded px-2.5 py-1.5 font-mono text-xs {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-zinc-900 text-zinc-100 dark:bg-zinc-950'}"
          >
            <span class={failed ? 'text-destructive/70' : 'text-zinc-500'}>$</span>
            {command}
          </div>
          {#if result !== null}
            <pre
              class="max-h-64 overflow-auto rounded p-2 text-xs {failed
                ? 'bg-destructive/10 text-destructive'
                : 'bg-background'}"><code>{failed ? failedOutput : result}</code></pre
            >
          {/if}
        </div>
      {:else if isListDir && result !== null && !failed}
        <ul class="grid max-h-64 grid-cols-2 gap-x-3 gap-y-0.5 overflow-auto text-xs sm:grid-cols-3">
          {#each dirEntries as entry}
            <li class="truncate font-mono text-muted-foreground">{entry}</li>
          {/each}
        </ul>
      {:else if isSearch && result !== null && !failed}
        <ul class="max-h-64 space-y-0.5 overflow-auto text-xs">
          {#each searchHits as hit}
            <li class="truncate font-mono text-muted-foreground">{hit}</li>
          {/each}
        </ul>
      {:else if failed && result !== null}
        <pre class="max-h-64 overflow-auto rounded bg-destructive/10 p-2 text-xs text-destructive"
          ><code>{failedOutput}</code></pre
        >
      {:else if isRead && result !== null}
        <pre
          class="max-h-64 overflow-auto rounded p-2 text-xs {failed
            ? 'bg-destructive/10 text-destructive'
            : 'bg-background'}"><code>{failed ? failedOutput : result}</code></pre
        >
      {:else if tool === "delegate_task"}
        <p class="text-xs text-muted-foreground">{String(args.goal ?? "")}</p>
        {#if result !== null}
          <pre
            class="max-h-64 overflow-auto rounded p-2 text-xs {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-background'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {/if}
      {:else}
        <dl class="space-y-1 text-xs">
          {#each Object.entries(args) as [key, value]}
            <div class="flex gap-2">
              <dt class="shrink-0 text-muted-foreground">{key}</dt>
              <dd class="min-w-0 truncate font-mono">{String(value)}</dd>
            </div>
          {/each}
        </dl>
        {#if result !== null}
          <pre
            class="max-h-64 overflow-auto rounded p-2 text-xs {failed
              ? 'bg-destructive/10 text-destructive'
              : 'bg-background'}"><code>{failed ? failedOutput : result}</code></pre
          >
        {/if}
      {/if}
    </div>
  {/if}
</div>
