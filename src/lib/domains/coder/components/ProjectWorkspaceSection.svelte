<script lang="ts">
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import Folder from '~icons/fa6-solid/folder';
  import Star from '~icons/fa6-solid/star';
  import Trash from '~icons/fa6-solid/trash';
  import { Button } from '$lib/components/ui/button';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import type { MenuItem } from '$lib/components/menu/types';
  import CoderSessionCard from './CoderSessionCard.svelte';
  import type { SessionCardBadge } from '$lib/domains/ai/components/shared/AISessionCard.svelte';
  import type { CoderThread } from '../types.js';
  import type { ProjectWorkspace } from '../utils/sessionList.js';

  interface Props {
    workspace: ProjectWorkspace;
    expanded?: boolean;
    active?: boolean;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    threads?: CoderThread[];
    onToggle?: () => void;
    onSelectProject?: () => void;
    onThreadClick?: (thread: CoderThread) => void;
    onDeleteThread?: (thread: CoderThread) => void;
    queuedCountFor?: (threadId: string) => number;
    subAgentSummaryFor?: (threadId: string) => { running: number; total: number };
    badgeFor?: (threadId: string) => SessionCardBadge[];
    onLaunchWorkspace?: ((thread: CoderThread) => void) | null;
    /** Right-click actions for the header. */
    menuItems?: MenuItem[];
    starred?: boolean;
    /** Delete every session of this workspace; undefined hides the button. */
    onDeleteHistory?: () => void;
    starredThreadIds?: string[];
    onToggleThreadStar?: (thread: CoderThread) => void;
  }

  let {
    workspace,
    expanded = false,
    active = false,
    selectedThreadId = null,
    runningThreadIds = new Set<string>(),
    threads = workspace.threads,
    onToggle,
    onSelectProject,
    onThreadClick,
    onDeleteThread,
    queuedCountFor,
    subAgentSummaryFor,
    badgeFor,
    onLaunchWorkspace,
    menuItems = [],
    starred = false,
    onDeleteHistory,
    starredThreadIds = [],
    onToggleThreadStar,
  }: Props = $props();

  const sessionCount = $derived(threads.length);
  const hasRunning = $derived(threads.some((t) => runningThreadIds.has(t.id)));
</script>

<div class="rounded-md {active ? 'bg-muted/50' : ''}">
  <ContextMenuList items={menuItems}>
    {#snippet trigger(menuProps)}
      <div
        {...menuProps}
        class="group flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm transition-colors hover:bg-muted/60"
      >
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          class="h-5 w-5 shrink-0"
          title={expanded ? 'Collapse' : 'Expand'}
          onclick={() => onToggle?.()}
        >
          <ChevronRight
            class="h-3.5 w-3.5 text-muted-foreground transition-transform {expanded
              ? 'rotate-90'
              : ''}"
          />
        </Button>
        <Button
          type="button"
          variant="ghost"
          class="h-auto min-w-0 flex-1 justify-start gap-1.5 px-0 text-left text-sm"
          onclick={() => {
            onSelectProject?.();
            if (!expanded) onToggle?.();
          }}
        >
          <Folder
            class="h-3.5 w-3.5 shrink-0 {active
              ? 'text-primary'
              : 'text-muted-foreground'} {workspace.projectId ? '' : 'opacity-40'}"
          />
          <span
            class="min-w-0 flex-1 truncate font-medium {active
              ? 'text-foreground'
              : 'text-foreground/90'}"
            title={workspace.projectId
              ? workspace.path
              : `${workspace.path} — folder not added as a project (right-click to add)`}
          >
            {workspace.label}
          </span>
          {#if starred}
            <Star class="h-3 w-3 shrink-0 text-amber-500" aria-label="Starred" />
          {/if}
          {#if hasRunning}
            <span
              class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-primary"
              title="Running session"
            ></span>
          {/if}
          {#if sessionCount > 0}
            <span class="shrink-0 text-xs text-muted-foreground tabular-nums">
              {sessionCount}
            </span>
          {/if}
        </Button>
        {#if onDeleteHistory}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            class="h-5 w-5 shrink-0 text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-destructive focus-visible:opacity-100"
            title="Delete chat history"
            aria-label="Delete chat history for {workspace.label}"
            onclick={onDeleteHistory}
          >
            <Trash class="h-3 w-3" />
          </Button>
        {/if}
      </div>
    {/snippet}
  </ContextMenuList>

  {#if expanded && threads.length > 0}
    <div class="space-y-1 pt-0.5 pr-1 pb-1 pl-3">
      {#each threads as t (t.id)}
        <div data-session-id={t.id}>
          <CoderSessionCard
            thread={t}
            hideProject
            onClick={() => onThreadClick?.(t)}
            onDelete={onDeleteThread}
            isActive={selectedThreadId === t.id}
            isRunning={runningThreadIds.has(t.id)}
            queuedCount={queuedCountFor?.(t.id) ?? 0}
            subAgentRunning={subAgentSummaryFor?.(t.id).running ?? 0}
            extraBadges={badgeFor?.(t.id)}
            {onLaunchWorkspace}
            starred={starredThreadIds.includes(t.id)}
            onToggleStar={onToggleThreadStar}
          />
        </div>
      {/each}
    </div>
  {:else if expanded && sessionCount === 0}
    <p class="px-3 pb-2 pl-8 text-xs text-muted-foreground">No sessions — start one below</p>
  {/if}
</div>
