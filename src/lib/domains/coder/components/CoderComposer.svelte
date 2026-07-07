<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import {
    Plus,
    ArrowUp,
    GitBranch,
    Monitor,
    ChevronDown,
    GitCompare,
    X,
    ListOrdered,
  } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import AIContextBar from "$lib/domains/ai/components/AIContextBar.svelte";
  import type { ContextUsage, LlmUsage } from "$lib/domains/ai/types/index.js";
  import { coderService } from "../services/coderService.js";
  import type { GitDiffStats, PermissionMode } from "../types.js";

  interface Props {
    value: string;
    onSend: () => void;
    placeholder?: string;
    disabled?: boolean;
    running?: boolean;
    mode: PermissionMode;
    onModeChange: (mode: PermissionMode) => void;
    workspaceRoot: string;
    onToggleChanges?: () => void;
    showChanges?: boolean;
    /** Bumps when agent file changes are refreshed (accept/reject). */
    changesRevision?: string;
    contextUsage?: ContextUsage | null;
    llmUsage?: LlmUsage | null;
    rows?: number;
    queuedMessages?: string[];
    onRemoveQueued?: (index: number) => void;
    class?: string;
  }

  let {
    value = $bindable(""),
    onSend,
    placeholder = "Send follow-up…",
    disabled = false,
    running = false,
    mode,
    onModeChange,
    workspaceRoot,
    onToggleChanges,
    showChanges = false,
    changesRevision = "",
    contextUsage = null,
    llmUsage = null,
    rows = 2,
    queuedMessages = [],
    onRemoveQueued,
    class: className = "",
  }: Props = $props();

  const MODES: { value: PermissionMode; label: string; hint: string }[] = [
    { value: "auto-accept-all", label: "Auto", hint: "Run everything automatically" },
    { value: "review", label: "Review", hint: "Prompt on each mutating action" },
    { value: "plan", label: "Plan", hint: "Read-only, no writes or commands" },
  ];

  let textareaEl = $state<HTMLTextAreaElement | null>(null);
  let gitStats = $state<GitDiffStats | null>(null);
  let wasRunning = $state(false);

  const modeLabel = $derived(MODES.find((m) => m.value === mode)?.label ?? "Auto");
  const workspaceName = $derived(
    workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ?? workspaceRoot,
  );
  const gitBranch = $derived(gitStats?.branch ?? null);
  const hasGitChanges = $derived(!!gitStats?.hasChanges);
  const followUpPlaceholder = $derived(
    running
      ? queuedMessages.length > 0
        ? `Queue follow-up (${queuedMessages.length} queued)…`
        : "Queue follow-up while agent runs…"
      : placeholder,
  );
  const queueCount = $derived(queuedMessages.length);

  onMount(() => {
    void refreshGitStats();
  });

  $effect(() => {
    workspaceRoot;
    void refreshGitStats();
  });

  $effect(() => {
    if (wasRunning && !running) {
      void refreshGitStats();
    }
    wasRunning = running;
  });

  $effect(() => {
    showChanges;
    changesRevision;
    void refreshGitStats();
  });

  async function refreshGitStats() {
    if (!workspaceRoot) {
      gitStats = null;
      return;
    }
    try {
      gitStats = await coderService.getGitDiffStats(workspaceRoot);
    } catch {
      gitStats = null;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter") return;
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      onSend();
    }
  }

  function autoResize() {
    if (!textareaEl) return;
    textareaEl.style.height = "auto";
    textareaEl.style.height = `${Math.min(textareaEl.scrollHeight, 160)}px`;
  }

  $effect(() => {
    value;
    autoResize();
  });
</script>

<div class={cn("px-4 pb-3 pt-2", className)}>
  {#if hasGitChanges && onToggleChanges}
    <div class="mx-auto mb-2 flex w-full max-w-3xl gap-2">
      <button
        type="button"
        onclick={onToggleChanges}
        class={cn(
          "inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium transition-colors",
          showChanges
            ? "border-primary/40 bg-primary/10 text-foreground"
            : "border-border bg-background text-foreground hover:bg-muted/60",
        )}
        title="Git changes in {gitBranch ?? 'workspace'}"
      >
        <GitCompare class="h-3.5 w-3.5 text-muted-foreground" />
        <span>Changes</span>
        {#if gitStats && gitStats.additions > 0}
          <span class="font-mono text-emerald-600 dark:text-emerald-400">
            +{gitStats.additions}
          </span>
        {/if}
        {#if gitStats && gitStats.deletions > 0}
          <span class="font-mono text-red-600 dark:text-red-400">
            -{gitStats.deletions}
          </span>
        {/if}
      </button>
    </div>
  {/if}

  <div class="mx-auto w-full max-w-3xl">
    {#if queueCount > 0}
      <div class="mb-2 space-y-1">
        <div class="flex items-center gap-1.5 px-1 text-[11px] font-medium text-muted-foreground">
          <ListOrdered class="h-3 w-3" />
          <span>{queueCount} queued</span>
        </div>
        {#each queuedMessages as msg, i}
          <div
            class="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/30 px-2.5 py-1.5 text-xs"
          >
            <span class="min-w-0 flex-1 truncate text-foreground/80">{msg}</span>
            {#if onRemoveQueued}
              <button
                type="button"
                class="shrink-0 rounded p-0.5 text-muted-foreground hover:bg-muted hover:text-foreground"
                title="Remove from queue"
                onclick={() => onRemoveQueued(i)}
              >
                <X class="h-3 w-3" />
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <div
      class="flex items-end gap-1 rounded-[26px] border border-border/80 bg-background px-2 py-2 shadow-sm ring-1 ring-black/[0.03] dark:ring-white/[0.04]"
    >
      <Button
        type="button"
        variant="ghost"
        size="icon"
        class="mb-0.5 h-8 w-8 shrink-0 rounded-full text-muted-foreground hover:bg-muted/80"
        title="Add context"
        disabled={disabled}
      >
        <Plus class="h-4 w-4" />
      </Button>

      <textarea
        bind:this={textareaEl}
        bind:value
        rows={rows}
        placeholder={followUpPlaceholder}
        {disabled}
        onkeydown={handleKeydown}
        oninput={autoResize}
        class="max-h-40 min-h-[36px] flex-1 resize-none border-0 bg-transparent px-1 py-1.5 text-sm leading-relaxed shadow-none outline-none placeholder:text-muted-foreground/70 focus-visible:ring-0"
      ></textarea>

      <div class="mb-0.5 flex shrink-0 items-center gap-1">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger
            class="inline-flex h-8 items-center gap-0.5 rounded-full px-2.5 text-xs text-muted-foreground transition-colors hover:bg-muted/80 hover:text-foreground"
          >
            {modeLabel}
            <ChevronDown class="h-3 w-3 opacity-60" />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" class="w-44">
            <DropdownMenu.RadioGroup value={mode} onValueChange={(v) => onModeChange(v as PermissionMode)}>
              {#each MODES as m}
                <DropdownMenu.RadioItem value={m.value}>
                  <div class="flex flex-col gap-0.5">
                    <span>{m.label}</span>
                    <span class="text-[10px] text-muted-foreground">{m.hint}</span>
                  </div>
                </DropdownMenu.RadioItem>
              {/each}
            </DropdownMenu.RadioGroup>
          </DropdownMenu.Content>
        </DropdownMenu.Root>

        <Button
          type="button"
          onclick={onSend}
          disabled={!value.trim() || disabled}
          size="icon"
          class="h-8 w-8 rounded-full"
          title={running ? "Queue message (Ctrl/Cmd+Enter)" : "Send (Ctrl/Cmd+Enter)"}
        >
          {#if running && value.trim()}
            <ListOrdered class="h-4 w-4" />
          {:else}
            <ArrowUp class="h-4 w-4" />
          {/if}
        </Button>
      </div>
    </div>

    <div class="mt-2 flex items-center justify-between gap-3 px-1 text-[11px] text-muted-foreground">
      <div class="flex min-w-0 items-center gap-3">
        <span class="inline-flex max-w-[120px] items-center gap-1 truncate" title={gitBranch ?? "No branch"}>
          <GitBranch class="h-3 w-3 shrink-0" />
          <span class="truncate">{gitBranch ?? "—"}</span>
        </span>
        <span
          class="inline-flex max-w-[160px] items-center gap-1 truncate"
          title={workspaceRoot || "No workspace"}
        >
          <Monitor class="h-3 w-3 shrink-0" />
          <span class="truncate">{workspaceName || "This PC"}</span>
        </span>
      </div>

      <AIContextBar variant="ring" {contextUsage} {llmUsage} />
    </div>
  </div>
</div>
