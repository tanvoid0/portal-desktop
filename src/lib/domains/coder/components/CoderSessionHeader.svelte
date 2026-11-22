<!-- One-row title bar above a coder feed, shared by every provider: title, run state, meta,
     chips and (when `onProjectSelect` is given) the project picker. Plain props only. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import CoderProjectSelector from './CoderProjectSelector.svelte';

  interface Props {
    title: string;
    /** Sub-agent breadcrumb prefix, with the way back to its coordinator. */
    subAgent?: boolean;
    running?: boolean;
    /** Replaces the default "Running" label next to the status dot. */
    statusLabel?: string;
    queuedCount?: number;
    /** Quiet secondary text, e.g. backend · model. */
    meta?: string;
    /** Total spend, shown as quiet meta. */
    costUsd?: number | null;
    workspaceRoot?: string;
    workspaceName?: string;
    /** Extra clickable chips (project / task links). */
    chips?: Snippet;
    onBackToCoordinator?: () => void;
    /** Present = the project is pickable here (locked via `projectLocked`). */
    onProjectSelect?: (path: string, projectId?: string) => void;
    projectLocked?: boolean;
  }

  let {
    title,
    subAgent = false,
    running = false,
    statusLabel,
    queuedCount = 0,
    meta,
    costUsd = null,
    workspaceRoot = $bindable(''),
    workspaceName = '',
    chips,
    onBackToCoordinator,
    onProjectSelect,
    projectLocked = false,
  }: Props = $props();

  const label = $derived(statusLabel ?? (running ? 'Running' : ''));
</script>

<div
  class="divider-edge-b divider-edge-full flex h-10 shrink-0 items-center gap-2 bg-background px-3"
>
  <div class="flex min-w-0 flex-1 items-center gap-2">
    <h2 class="min-w-0 truncate text-[13px] font-medium">
      {#if subAgent}
        <span class="text-muted-foreground">Sub-agent</span>
        <span class="mx-1 text-muted-foreground">/</span>
      {/if}
      {title}
    </h2>
    {#if subAgent && onBackToCoordinator}
      <Button
        size="sm"
        variant="ghost"
        class="h-7 shrink-0 gap-1 text-xs"
        title="Back to coordinator"
        onclick={onBackToCoordinator}
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Coordinator
      </Button>
    {/if}
    {#if label}
      <span class="flex shrink-0 items-center gap-1 text-xs text-muted-foreground">
        <span
          class={[
            'h-1.5 w-1.5 rounded-full',
            running ? 'animate-pulse bg-primary' : 'bg-muted-foreground/50',
          ]}
        ></span>
        {label}
      </span>
    {/if}
    {#if queuedCount > 0}
      <span class="shrink-0 text-xs text-muted-foreground">{queuedCount} queued</span>
    {/if}
    {#if meta}
      <span class="hidden min-w-0 shrink truncate text-xs text-muted-foreground sm:inline">
        {meta}
      </span>
    {/if}
    {#if costUsd != null}
      <span class="shrink-0 text-xs text-muted-foreground tabular-nums">
        ${costUsd.toFixed(4)}
      </span>
    {/if}
    {#if workspaceRoot && !onProjectSelect}
      <span
        class="hidden min-w-0 shrink items-center gap-1 text-xs text-muted-foreground sm:flex"
        title={workspaceRoot}
      >
        <FolderOpen class="h-3 w-3 shrink-0" />
        <span class="max-w-[200px] truncate">{workspaceName}</span>
      </span>
    {/if}
    {@render chips?.()}
  </div>

  {#if onProjectSelect}
    <div class="flex w-44 min-w-0 shrink items-center gap-1.5 sm:w-64">
      <CoderProjectSelector
        bind:value={workspaceRoot}
        disabled={projectLocked}
        onSelect={onProjectSelect}
      />
    </div>
  {/if}
</div>
