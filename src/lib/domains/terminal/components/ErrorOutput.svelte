<!--
  ErrorOutput — friendly rendering of failed command output with a prominent
  Ask AI action. Falls back to raw text when parsing yields nothing useful.
-->
<script lang="ts">
  import * as Alert from "$lib/components/ui/alert";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    AlertCircle,
    ChevronDown,
    ChevronRight,
    Lightbulb,
    Sparkles,
  } from "@lucide/svelte";
  import {
    parseTerminalError,
    ERROR_CATEGORY_LABELS,
  } from "../utils/parseTerminalError";

  interface Props {
    output: string;
    command?: string;
    exitCode?: number;
    onAskAI?: () => void;
  }

  let { output, command, exitCode, onAskAI }: Props = $props();

  let showRaw = $state(false);

  const parsed = $derived(parseTerminalError(output, command));
</script>

{#if parsed}
  <div class="space-y-3 px-3 py-3">
    <Alert.Root variant="destructive" class="border-status-error/40 bg-status-error-bg/30">
      <AlertCircle class="text-status-error" />
      <Alert.Title class="flex flex-wrap items-center gap-2 text-foreground">
        <span>{parsed.title}</span>
        <Badge variant="outline" class="text-[10px] text-status-error">
          {ERROR_CATEGORY_LABELS[parsed.category]}
        </Badge>
        {#if exitCode !== undefined && exitCode !== 0}
          <Badge variant="destructive" class="text-[10px]">
            exit {exitCode}
          </Badge>
        {/if}
      </Alert.Title>
      <Alert.Description class="text-foreground/90">
        <p class="text-sm leading-relaxed">{parsed.message}</p>
        {#if parsed.hint}
          <p class="mt-2 flex items-start gap-1.5 text-xs text-muted-foreground">
            <Lightbulb class="mt-0.5 h-3.5 w-3.5 shrink-0 text-status-warning" />
            <span>{parsed.hint}</span>
          </p>
        {/if}
      </Alert.Description>
    </Alert.Root>

    {#if parsed.details.length > 0}
      <ul class="space-y-0.5 pl-1 font-mono text-xs text-muted-foreground">
        {#each parsed.details.slice(0, 4) as line}
          <li class="truncate" title={line}>{line}</li>
        {/each}
      </ul>
    {/if}

    <div class="flex flex-wrap items-center gap-2">
      {#if onAskAI}
        <Button
          variant="default"
          size="sm"
          class="gap-1.5"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            onAskAI();
          }}
        >
          <Sparkles class="h-3.5 w-3.5" />
          Ask AI to fix this
        </Button>
      {/if}
      <Button
        variant="ghost"
        size="sm"
        class="gap-1 text-xs text-muted-foreground"
        onclick={(e: MouseEvent) => {
          e.stopPropagation();
          showRaw = !showRaw;
        }}
      >
        {#if showRaw}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight class="h-3 w-3" />{/if}
        {showRaw ? "Hide" : "Show"} raw output
      </Button>
    </div>

    {#if showRaw}
      <pre
        class="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-md border border-border bg-muted/30 p-2 font-mono text-xs leading-relaxed text-foreground/80"
      >{parsed.raw}</pre>
    {/if}
  </div>
{:else}
  <pre
    class="max-h-96 overflow-auto whitespace-pre-wrap break-words bg-muted/20 px-3 py-2 font-mono text-xs leading-relaxed text-foreground/90"
  >{output || " "}</pre>
{/if}
