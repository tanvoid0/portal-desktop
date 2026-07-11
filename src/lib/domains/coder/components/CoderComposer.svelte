<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import {
    Plus,
    ArrowUp,
    Bot,
    GitBranch,
    GitBranchPlus,
    Monitor,
    ChevronDown,
    X,
    ListOrdered,
    Square,
  } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import AIContextBar from "$lib/domains/ai/components/AIContextBar.svelte";
  import type { ContextUsage, LlmUsage } from "$lib/domains/ai/types/index.js";
  import { coderService } from "../services/coderService.js";
  import type { GitDiffStats, PermissionMode } from "../types.js";

  interface Props {
    value: string;
    onSend: () => void;
    onStop?: () => void;
    placeholder?: string;
    disabled?: boolean;
    running?: boolean;
    mode: PermissionMode;
    onModeChange: (mode: PermissionMode) => void;
    workspaceRoot: string;
    onToggleChanges?: () => void;
    showChanges?: boolean;
    onToggleGitChanges?: () => void;
    showGitChanges?: boolean;
    pendingChangeCount?: number;
    /** Bumps when agent file changes are refreshed (accept/reject). */
    changesRevision?: string;
    contextUsage?: ContextUsage | null;
    llmUsage?: LlmUsage | null;
    rows?: number;
    queuedMessages?: string[];
    onRemoveQueued?: (index: number) => void;
    multitaskMode?: boolean;
    onToggleMultitask?: (enabled: boolean) => void;
    class?: string;
  }

  let {
    value = $bindable(""),
    onSend,
    onStop,
    placeholder = "Send follow-up…",
    disabled = false,
    running = false,
    mode,
    onModeChange,
    workspaceRoot,
    onToggleChanges,
    showChanges = false,
    onToggleGitChanges,
    showGitChanges = false,
    pendingChangeCount = 0,
    changesRevision = "",
    contextUsage = null,
    llmUsage = null,
    rows = 2,
    queuedMessages = [],
    onRemoveQueued,
    multitaskMode = false,
    onToggleMultitask,
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
  const hasAgentChanges = $derived(pendingChangeCount > 0);
  const followUpPlaceholder = $derived(
    running
      ? queuedMessages.length > 0
        ? `Queue follow-up (${queuedMessages.length} queued)…`
        : "Queue follow-up while agent runs…"
      : multitaskMode
        ? "Paste 2+ GitHub issue URLs to spawn parallel worktrees…"
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
  {#if (hasAgentChanges && onToggleChanges) || (hasGitChanges && onToggleGitChanges)}
    <div class="mx-auto mb-2 flex w-full max-w-3xl flex-wrap gap-2 lg:max-w-4xl xl:max-w-5xl 2xl:max-w-6xl">
      {#if hasAgentChanges && onToggleChanges}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onclick={onToggleChanges}
          class={cn(
            "h-auto rounded-full px-3 py-1 text-xs font-medium",
            showChanges
              ? "border-primary/40 bg-primary/10 text-foreground"
              : "border-border bg-background text-foreground hover:bg-muted/60",
          )}
          title="Review agent file changes"
        >
          <Bot class="h-3.5 w-3.5 text-muted-foreground" />
          <span>Agent changes</span>
          <span class="rounded-full bg-amber-500 px-1.5 font-mono text-[10px] text-white">
            {pendingChangeCount}
          </span>
        </Button>
      {/if}
      {#if hasGitChanges && onToggleGitChanges}
        <Button
          type="button"
          variant="outline"
          size="sm"
          onclick={onToggleGitChanges}
          class={cn(
            "h-auto rounded-full px-3 py-1 text-xs font-medium",
            showGitChanges
              ? "border-primary/40 bg-primary/10 text-foreground"
              : "border-border bg-background text-foreground hover:bg-muted/60",
          )}
          title="Git changes in {gitBranch ?? 'workspace'}"
        >
          <GitBranch class="h-3.5 w-3.5 text-muted-foreground" />
          <span>Git changes</span>
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
        </Button>
      {/if}
    </div>
  {/if}

  {#if onToggleMultitask}
    <div class="mx-auto mb-2 flex w-full max-w-3xl lg:max-w-4xl xl:max-w-5xl 2xl:max-w-6xl">
      <Button
        type="button"
        variant="outline"
        size="sm"
        onclick={() => onToggleMultitask(!multitaskMode)}
        class={cn(
          "h-auto rounded-full px-3 py-1 text-xs font-medium",
          multitaskMode
            ? "border-primary/40 bg-primary/10 text-foreground"
            : "border-border bg-background text-foreground hover:bg-muted/60",
        )}
        title="Use a coordinator thread that can fan out parallel worktree agents"
      >
        <GitBranchPlus class="h-3.5 w-3.5 text-muted-foreground" />
        <span>{multitaskMode ? "Multitask on" : "Multitask off"}</span>
      </Button>
    </div>
  {/if}

  <div class="mx-auto w-full max-w-3xl lg:max-w-4xl xl:max-w-5xl 2xl:max-w-6xl">
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
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                class="h-5 w-5 shrink-0"
                title="Remove from queue"
                onclick={() => onRemoveQueued(i)}
              >
                <X class="h-3 w-3" />
              </Button>
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

      <Textarea
        bind:ref={textareaEl}
        bind:value
        rows={rows}
        placeholder={followUpPlaceholder}
        {disabled}
        onkeydown={handleKeydown}
        oninput={autoResize}
        class="max-h-40 min-h-[36px] flex-1 resize-none border-0 bg-transparent px-1 py-1.5 text-sm leading-relaxed shadow-none outline-none placeholder:text-muted-foreground/70 focus-visible:ring-0"
      />

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

        {#if running && !value.trim() && onStop}
          <Button
            type="button"
            onclick={onStop}
            size="icon"
            variant="destructive"
            class="h-8 w-8 rounded-full"
            title="Stop agent"
          >
            <Square class="h-3.5 w-3.5 fill-current" />
          </Button>
        {:else}
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
        {/if}
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
