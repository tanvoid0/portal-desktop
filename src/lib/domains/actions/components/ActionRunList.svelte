<!--
  Runs list (RunSummary rows: actions, pipelines, releases). Click selects; right-click
  opens the run's menu — open, stop, and for actions re-run, run in terminal, copy,
  open project, delete.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import type { MenuItem } from '$lib/components/menu/types';
  import Eye from '~icons/fa6-solid/eye';
  import OpenPage from '~icons/fa6-solid/up-right-from-square';
  import { goto } from '$app/navigation';
  import Stop from '~icons/fa6-solid/stop';
  import Rerun from '~icons/fa6-solid/rotate-right';
  import TerminalIcon from '~icons/fa6-solid/terminal';
  import Copy from '~icons/fa6-solid/copy';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Trash from '~icons/fa6-solid/trash';
  import Robot from '~icons/fa6-solid/robot';
  import {
    scriptExecutionService,
    type ScriptExecutionInfo,
  } from '$lib/domains/scripts/services/scriptExecutionService';
  import RunProgressBar from './RunProgressBar.svelte';
  import type { RunSummary } from '../types';
  import {
    formatExecutionDuration,
    getExecutionStatusBadgeClass,
    getExecutionStatusBadgeVariant,
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from '$lib/domains/projects/pipelines/utils/executionDisplay';
  import { copyRunText, deleteRun, isActive, rerun, stopRun } from '../runActions';
  import { runRequester } from '../runOutput';

  interface Props {
    executions: RunSummary[];
    selectedId?: string | null;
    /** Project label per row (the global list); omit inside a project. */
    projectName?: (exec: RunSummary) => string | null;
    onSelect: (exec: RunSummary, opts?: { terminal?: boolean }) => void;
    /** A new run started from the menu (Re-run). */
    onStarted?: (executionId: string) => void;
    /** A run was stopped or deleted (then its id is passed) — reload. */
    onChanged?: (deletedId?: string) => void;
    onOpenProject?: (exec: RunSummary) => void;
  }

  let {
    executions,
    selectedId = null,
    projectName,
    onSelect,
    onStarted,
    onChanged,
    onOpenProject,
  }: Props = $props();

  /** Re-run / copy need the command, which the summary row does not carry. */
  const withFull = (id: string, fn: (e: ScriptExecutionInfo) => unknown) => async () => {
    const full = await scriptExecutionService.getExecution(id);
    if (full) await fn(full);
  };

  function menuFor(exec: RunSummary): MenuItem[] {
    const active = isActive(exec);
    const isAction = exec.kind === 'action';
    const items: MenuItem[] = [
      { label: 'Open details', icon: Eye, onSelect: () => onSelect(exec) },
    ];
    // ponytail: non-action rows have no standalone page unless the run knows its project.
    if (isAction || exec.projectId) {
      items.push({
        label: 'Open page',
        icon: OpenPage,
        onSelect: () =>
          void goto(
            isAction
              ? `/pipeline-runs/${exec.id}`
              : `/projects/${exec.projectId}/pipelines/run/${exec.id}`
          ),
      });
    }
    if (active) {
      items.push({
        label: 'Stop',
        icon: Stop,
        variant: 'destructive',
        onSelect: async () => {
          if (await stopRun(exec)) onChanged?.();
        },
      });
    } else if (isAction) {
      items.push({
        label: 'Re-run',
        icon: Rerun,
        onSelect: withFull(exec.id, async (full) => {
          const id = await rerun(full);
          if (id) onStarted?.(id);
        }),
      });
    }
    if (isAction) {
      items.push(
        {
          label: 'Run in terminal',
          icon: TerminalIcon,
          onSelect: () => onSelect(exec, { terminal: true }),
        },
        {
          label: 'Copy command',
          icon: Copy,
          separatorBefore: true,
          onSelect: withFull(exec.id, (full) => copyRunText('command', full)),
        },
        {
          label: 'Copy output',
          icon: Copy,
          onSelect: withFull(exec.id, (full) => copyRunText('output', full)),
        }
      );
    }
    if (onOpenProject && projectName?.(exec)) {
      items.push({
        label: 'Open project',
        icon: FolderOpen,
        onSelect: () => onOpenProject(exec),
      });
    }
    if (isAction)
      items.push({
        label: 'Delete',
        icon: Trash,
        variant: 'destructive',
        separatorBefore: true,
        disabled: active,
        onSelect: async () => {
          if (await deleteRun(exec)) onChanged?.(exec.id);
        },
      });
    return items;
  }
</script>

<div class="space-y-2">
  {#each executions as execution, i (execution.id)}
    {#if execution.workflowRunId && execution.workflowRunId !== executions[i - 1]?.workflowRunId}
      <p class="px-1 pt-2 text-xs font-medium text-muted-foreground">Workflow run</p>
    {/if}
    {@const StatusIcon = getExecutionStatusIcon(execution.status)}
    {@const requester = runRequester(execution.triggeredBy)}
    {@const project = projectName?.(execution)}
    <ContextMenuList items={menuFor(execution)}>
      {#snippet trigger(menuProps)}
        <Button
          {...menuProps}
          type="button"
          variant="outline"
          class="h-auto w-full min-w-0 justify-start overflow-hidden rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {selectedId ===
          execution.id
            ? 'border-primary bg-primary/5'
            : ''}"
          onclick={() => onSelect(execution)}
        >
          <div class="flex w-full min-w-0 flex-col gap-1">
            <div class="flex w-full flex-wrap items-center justify-between gap-2">
              <div class="flex min-w-0 items-center gap-2">
                <StatusIcon
                  class="h-4 w-4 shrink-0 {getExecutionStatusColor(execution.status)}{isActive(
                    execution
                  )
                    ? ' animate-spin'
                    : ''}"
                />
                <Badge
                  variant={getExecutionStatusBadgeVariant(execution.status)}
                  class={getExecutionStatusBadgeClass(execution.status)}
                >
                  {execution.status}
                </Badge>
                {#if project}
                  <span class="max-w-[40%] min-w-0 truncate text-sm font-semibold">{project}</span>
                {/if}
                {#if execution.kind !== 'action'}
                  <Badge variant="outline" class="capitalize">{execution.kind}</Badge>
                {/if}
                <span class="truncate text-sm font-medium">{execution.title}</span>
              </div>
              <span class="text-xs text-muted-foreground">
                {new Date(execution.startedAt).toLocaleString()}
              </span>
            </div>
            <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <span>
                Duration: {formatExecutionDuration(execution.startedAt, execution.finishedAt)}
              </span>
              {#if requester}
                <span class="flex items-center gap-1"><Robot class="h-3 w-3" />{requester}</span>
              {/if}
            </div>
            {#if execution.progress && isActive(execution)}
              <RunProgressBar progress={execution.progress} compact />
            {/if}
          </div>
        </Button>
      {/snippet}
    </ContextMenuList>
  {/each}
</div>
