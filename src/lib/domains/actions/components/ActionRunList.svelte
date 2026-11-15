<!--
  Action runs list (script_executions rows). Click selects; right-click opens the
  run's menu — open, stop, re-run, run in terminal, copy, open project, delete.
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
  import type { ScriptExecutionInfo } from '$lib/domains/scripts/services/scriptExecutionService';
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
    executions: ScriptExecutionInfo[];
    selectedId?: string | null;
    /** Project label per row (the global list); omit inside a project. */
    projectName?: (exec: ScriptExecutionInfo) => string | null;
    onSelect: (exec: ScriptExecutionInfo, opts?: { terminal?: boolean }) => void;
    /** A new run started from the menu (Re-run). */
    onStarted?: (executionId: string) => void;
    /** A run was stopped or deleted (then its id is passed) — reload. */
    onChanged?: (deletedId?: string) => void;
    onOpenProject?: (exec: ScriptExecutionInfo) => void;
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

  function menuFor(exec: ScriptExecutionInfo): MenuItem[] {
    const active = isActive(exec);
    const items: MenuItem[] = [
      { label: 'Open details', icon: Eye, onSelect: () => onSelect(exec) },
      {
        label: 'Open page',
        icon: OpenPage,
        onSelect: () => void goto(`/pipeline-runs/${exec.id}`),
      },
      active
        ? {
            label: 'Stop',
            icon: Stop,
            variant: 'destructive',
            onSelect: async () => {
              if (await stopRun(exec)) onChanged?.();
            },
          }
        : {
            label: 'Re-run',
            icon: Rerun,
            onSelect: async () => {
              const id = await rerun(exec);
              if (id) onStarted?.(id);
            },
          },
      {
        label: 'Run in terminal',
        icon: TerminalIcon,
        onSelect: () => onSelect(exec, { terminal: true }),
      },
      {
        label: 'Copy command',
        icon: Copy,
        separatorBefore: true,
        onSelect: () => void copyRunText('command', exec),
      },
      { label: 'Copy output', icon: Copy, onSelect: () => void copyRunText('output', exec) },
    ];
    if (onOpenProject && projectName?.(exec)) {
      items.push({
        label: 'Open project',
        icon: FolderOpen,
        onSelect: () => onOpenProject(exec),
      });
    }
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
  {#each executions as execution (execution.id)}
    {@const StatusIcon = getExecutionStatusIcon(execution.status)}
    {@const requester = runRequester(execution.triggeredBy)}
    {@const project = projectName?.(execution)}
    <ContextMenuList items={menuFor(execution)}>
      {#snippet trigger(menuProps)}
        <Button
          {...menuProps}
          type="button"
          variant="outline"
          class="h-auto w-full justify-start rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {selectedId ===
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
                <span class="truncate text-sm font-medium">{execution.command}</span>
              </div>
              <span class="text-xs text-muted-foreground">
                {new Date(execution.startedAt).toLocaleString()}
              </span>
            </div>
            <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <span>
                Duration: {formatExecutionDuration(execution.startedAt, execution.finishedAt)}
              </span>
              {#if execution.exitCode != null}
                <span>exit {execution.exitCode}</span>
              {/if}
              {#if requester}
                <span class="flex items-center gap-1"><Robot class="h-3 w-3" />{requester}</span>
              {/if}
              {#if execution.error}
                <span class="truncate text-destructive">{execution.error}</span>
              {/if}
            </div>
          </div>
        </Button>
      {/snippet}
    </ContextMenuList>
  {/each}
</div>
