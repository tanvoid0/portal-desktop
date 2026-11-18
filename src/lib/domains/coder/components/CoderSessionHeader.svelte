<!-- Title row above the Coder feed: session title, run state, workspace and project picker. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import PanelLeftOpen from '~icons/fa6-solid/table-columns';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import { coderSession } from '../state/coderSession.svelte.js';
  import type { CoderThread } from '../types.js';
  import CoderProjectSelector from './CoderProjectSelector.svelte';

  interface Props {
    thread: CoderThread | null | undefined;
    parentCoordinatorId: string | null | undefined;
    running: boolean;
    queuedCount: number;
    workspaceName: string;
    onOpenSessions: () => void;
    onBackToCoordinator: () => void;
    onProjectSelect: (path: string, projectId?: string) => void;
  }

  let {
    thread,
    parentCoordinatorId,
    running,
    queuedCount,
    workspaceName,
    onOpenSessions,
    onBackToCoordinator,
    onProjectSelect,
  }: Props = $props();
</script>

<div
  class="divider-edge-b divider-edge-full flex shrink-0 flex-wrap items-center justify-between gap-2 bg-background px-4 py-2.5"
>
  <div class="flex min-w-0 flex-1 items-center gap-2">
    <Button
      size="icon"
      variant="ghost"
      class="h-8 w-8 shrink-0 md:hidden"
      title="Sessions"
      onclick={onOpenSessions}
    >
      <PanelLeftOpen class="h-4 w-4" />
    </Button>
    <h2 class="truncate text-sm font-semibold">
      {#if thread?.thread_kind === 'sub-agent' && parentCoordinatorId}
        <span class="text-muted-foreground">Sub-agent</span>
        <span class="mx-1 text-muted-foreground">/</span>
      {/if}
      {thread?.title || 'New session'}
    </h2>
    {#if thread?.thread_kind === 'sub-agent' && parentCoordinatorId}
      <Button
        size="sm"
        variant="ghost"
        class="h-7 gap-1 text-xs"
        title="Back to coordinator"
        onclick={onBackToCoordinator}
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Coordinator
      </Button>
    {/if}
    {#if running}
      <span class="flex items-center gap-1 text-xs text-primary">
        <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-primary"></span>
        Running
      </span>
    {/if}
    {#if queuedCount > 0}
      <span class="text-xs text-muted-foreground">
        {queuedCount} queued
      </span>
    {/if}
    {#if coderSession.workspaceRoot}
      <span class="hidden items-center gap-1 text-xs text-muted-foreground sm:flex">
        <FolderOpen class="h-3 w-3" />
        <span class="max-w-[200px] truncate">{workspaceName}</span>
      </span>
    {/if}
  </div>

  <div class="flex w-full min-w-0 items-center gap-1.5 sm:w-auto sm:max-w-md sm:shrink">
    <CoderProjectSelector
      bind:value={coderSession.workspaceRoot}
      disabled={!!thread}
      onSelect={onProjectSelect}
    />
  </div>
</div>
