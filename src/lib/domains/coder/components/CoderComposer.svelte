<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { onMount, type Snippet } from 'svelte';

  import { Button } from '$lib/components/ui/button';

  import Plus from '~icons/fa6-solid/plus';
  import Bot from '~icons/fa6-solid/robot';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import X from '~icons/fa6-solid/xmark';
  import ListOrdered from '~icons/fa6-solid/list-ol';

  import { cn } from '$lib/utils.js';

  import AIComposerShell from '$lib/domains/ai/components/chat/AIComposerShell.svelte';

  import AIContextBar from '$lib/domains/ai/components/AIContextBar.svelte';

  import CoderAgentModeSelector from './CoderAgentModeSelector.svelte';
  import CoderPermissionModeSelector from './CoderPermissionModeSelector.svelte';
  import { nextAgentMode } from '../config/agentModes.js';
  import type { ContextUsage, LlmUsage } from '$lib/domains/ai/types/index.js';
  import { coderService } from '../services/coderService.js';
  import type { CoderAgentMode, GitDiffStats, PermissionMode } from '../types.js';

  interface Props {
    value: string;

    onSend: () => void;

    onStop?: () => void;

    placeholder?: string;

    disabled?: boolean;

    running?: boolean;

    /** "A live run exists" — defaults to `running`; pass `running || !!pending`
     *  so Stop stays reachable while paused on an approval banner (QA-119). */
    canStop?: boolean;

    /** Mode/permission chips render only when their value and handler are both given. */
    agentMode?: CoderAgentMode;

    effectiveMode?: CoderAgentMode | null;

    onAgentModeChange?: (mode: CoderAgentMode) => void;
    permissionMode?: PermissionMode;
    onPermissionModeChange?: (mode: PermissionMode) => void;
    /** Provider-specific model control, rendered after the mode chips. */
    model?: Snippet;
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

    /** Whether a message sent mid-run queues behind it (else it goes straight to the run). */
    queue?: boolean;

    onRemoveQueued?: (index: number) => void;

    class?: string;
  }

  let {
    value = $bindable(''),

    onSend,

    onStop,

    placeholder = 'Send follow-up…',

    disabled = false,

    running = false,

    canStop = running,

    agentMode,

    effectiveMode = null,

    onAgentModeChange,

    permissionMode,

    onPermissionModeChange,

    model,

    workspaceRoot,

    onToggleChanges,

    showChanges = false,

    onToggleGitChanges,

    showGitChanges = false,

    pendingChangeCount = 0,

    changesRevision = '',

    contextUsage = null,

    llmUsage = null,

    rows = 2,

    queuedMessages = [],

    queue = true,

    onRemoveQueued,

    class: className = '',
  }: Props = $props();

  let gitStats = $state<GitDiffStats | null>(null);

  let wasRunning = $state(false);

  const workspaceName = $derived(
    workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ?? workspaceRoot
  );

  const gitBranch = $derived(gitStats?.branch ?? null);

  const hasGitChanges = $derived(!!gitStats?.hasChanges);

  const hasAgentChanges = $derived(pendingChangeCount > 0);

  const followUpPlaceholder = $derived(
    running
      ? !queue
        ? 'Send follow-up…'
        : queuedMessages.length > 0
          ? `Queue follow-up (${queuedMessages.length} queued)…`
          : agentMode === 'multitask'
            ? 'Queue follow-up… (2+ lines spawns parallel agents)'
            : 'Queue follow-up…'
      : agentMode === 'multitask'
        ? 'One task per line (2+ lines), or paste 2+ GitHub issue URLs…'
        : placeholder
  );

  const queueCount = $derived(queuedMessages.length);

  function cycleAgentMode() {
    if (agentMode && onAgentModeChange) onAgentModeChange(nextAgentMode(agentMode));
  }

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
</script>

<AIComposerShell
  bind:value
  {onSend}
  {onStop}
  placeholder={followUpPlaceholder}
  {disabled}
  {running}
  {canStop}
  {rows}
  submitOn="modifier-enter"
  queueSend={queue}
  onModeCycle={cycleAgentMode}
  class={className}
>
  {#snippet above()}
    {#if (hasAgentChanges && onToggleChanges) || (hasGitChanges && onToggleGitChanges)}
      <div class="mb-2 flex flex-wrap gap-2">
        {#if hasAgentChanges && onToggleChanges}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onclick={onToggleChanges}
            class={cn(
              'h-auto rounded-full px-3 py-1 text-xs font-medium',

              showChanges
                ? 'border-primary/40 bg-primary/10 text-foreground'
                : 'border-border bg-background text-foreground hover:bg-muted/60'
            )}
            title="Review agent file changes"
          >
            <Bot class="h-3.5 w-3.5 text-muted-foreground" />

            <span>Agent changes</span>

            <Badge class="border-0 bg-amber-500 px-1.5 py-0 font-mono text-white">
              {pendingChangeCount}
            </Badge>
          </Button>
        {/if}

        {#if hasGitChanges && onToggleGitChanges}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onclick={onToggleGitChanges}
            class={cn(
              'h-auto rounded-full px-3 py-1 text-xs font-medium',

              showGitChanges
                ? 'border-primary/40 bg-primary/10 text-foreground'
                : 'border-border bg-background text-foreground hover:bg-muted/60'
            )}
            title="Git changes in {gitBranch ?? 'workspace'}"
          >
            <GitBranch class="h-3.5 w-3.5 text-muted-foreground" />

            <span>Git changes</span>

            {#if gitStats && gitStats.additions > 0}
              <span class="font-mono text-diff-add-foreground">
                +{gitStats.additions}
              </span>
            {/if}

            {#if gitStats && gitStats.deletions > 0}
              <span class="font-mono text-diff-del-foreground">
                -{gitStats.deletions}
              </span>
            {/if}
          </Button>
        {/if}
      </div>
    {/if}

    {#if queueCount > 0}
      <div class="mb-2 space-y-1">
        <div class="flex items-center gap-1.5 px-1 text-xs font-medium text-muted-foreground">
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
  {/snippet}

  {#snippet leading()}
    <Button
      type="button"
      variant="ghost"
      size="icon"
      class="mb-0.5 h-8 w-8 shrink-0 rounded-full text-muted-foreground hover:bg-muted/80"
      title="Add context"
      {disabled}
    >
      <Plus class="h-4 w-4" />
    </Button>
  {/snippet}

  {#snippet trailing()}
    {#if permissionMode && onPermissionModeChange}
      <CoderPermissionModeSelector
        mode={permissionMode}
        onModeChange={onPermissionModeChange}
        inert={agentMode === 'ask'}
        {disabled}
      />
    {/if}

    {#if agentMode && onAgentModeChange}
      <CoderAgentModeSelector
        mode={agentMode}
        {effectiveMode}
        onModeChange={onAgentModeChange}
        {disabled}
      />
    {/if}

    {@render model?.()}
  {/snippet}

  {#snippet footer()}
    <div class="flex items-center justify-between gap-3 px-1 text-xs text-muted-foreground">
      <div class="flex min-w-0 items-center gap-3">
        <span
          class="inline-flex max-w-[120px] items-center gap-1 truncate"
          title={gitBranch ?? 'No branch'}
        >
          <GitBranch class="h-3 w-3 shrink-0" />

          <span class="truncate">{gitBranch ?? '—'}</span>
        </span>

        <span
          class="inline-flex max-w-[160px] items-center gap-1 truncate"
          title={workspaceRoot || 'No workspace'}
        >
          <span class="truncate">{workspaceName || 'This PC'}</span>
        </span>

        <span class="truncate">
          Ctrl+Enter · send{agentMode && onAgentModeChange ? ' · Shift+Tab · cycle mode' : ''}
        </span>
      </div>

      <AIContextBar variant="ring" {contextUsage} {llmUsage} />
    </div>
  {/snippet}
</AIComposerShell>
