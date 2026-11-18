<script lang="ts">
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaClock from '~icons/fa6-solid/clock';
  import FaLock from '~icons/fa6-solid/lock';
  import FaUser from '~icons/fa6-solid/user';
  import FaFolder from '~icons/fa6-solid/folder';
  import FaCircleExclamation from '~icons/fa6-solid/circle-exclamation';
  import FaCalendarWeek from '~icons/fa6-solid/calendar-week';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import {
    handleDragStart,
    handleDragEnd,
    getTaskCardClasses,
    type DragDropState,
  } from '../utils/dragDrop';
  import {
    isOverdue,
    isDueToday,
    isDueThisWeek,
    formatMinutes,
    type TaskDisplay,
  } from '../utils/taskDisplay';
  import type { Task } from '../types';
  import type { buildTaskMenu } from '../utils/taskMenu';
  import { stripMarkdown } from '../utils/stripMarkdown';

  interface Props {
    task: Task;
    menuItems: ReturnType<typeof buildTaskMenu>;
    display: TaskDisplay;
    subCount: number;
    /** Project name to show (All-projects mode); undefined hides it. */
    project?: string;
    selected: boolean;
    multiSelectMode: boolean;
    readonly: boolean;
    showSubtaskActions: boolean;
    dragDropState: DragDropState;
    onActivate: () => void;
    onToggleSelection: () => void;
    onProjectClick?: () => void;
    onCreateSubtask: () => void;
    onToggleStatus: () => void;
  }
  let {
    task,
    menuItems,
    display,
    subCount,
    project,
    selected,
    multiSelectMode,
    readonly,
    showSubtaskActions,
    dragDropState,
    onActivate,
    onToggleSelection,
    onProjectClick,
    onCreateSubtask,
    onToggleStatus,
  }: Props = $props();

  const tags = $derived(task.tags ?? []);
  const blocked = $derived(!!task.blockedBy && task.blockedBy.length > 0);
</script>

<ContextMenuList items={menuItems}>
  {#snippet trigger(menuProps)}
    <Card
      {...menuProps}
      draggable={readonly ? undefined : 'true'}
      ondragstart={readonly ? undefined : (e: DragEvent) => handleDragStart(e, task, dragDropState)}
      ondragend={readonly ? undefined : (e: DragEvent) => handleDragEnd(e, dragDropState)}
      onclick={onActivate}
      onkeydown={(e: KeyboardEvent) => {
        if (e.target !== e.currentTarget) return;
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onActivate();
        }
      }}
      role="group"
      tabindex={0}
      aria-label="Task: {task.title}"
      aria-current={selected ? 'true' : undefined}
      class="cursor-pointer gap-0 py-0 hover:shadow-md {selected
        ? 'bg-warning-50 ring-2 ring-warning-500 dark:bg-warning-900/20'
        : ''} {getTaskCardClasses(task, dragDropState)}"
    >
      <CardContent class="space-y-2 p-3">
        <div class="flex items-start gap-2">
          {#if multiSelectMode}
            <Checkbox
              checked={selected}
              onCheckedChange={onToggleSelection}
              onclick={(e) => e.stopPropagation()}
              class="mt-0.5 shrink-0"
            />
          {/if}
          <h4 class="line-clamp-2 min-w-0 flex-1 text-sm font-medium break-words text-foreground">
            {task.title}
          </h4>
          <div class="flex shrink-0 items-center pt-0.5">
            <span title="{task.priority} priority" aria-label="{task.priority} priority">
              <FaFlag class="h-3 w-3 {display.priorityColor(task.priority)}" />
            </span>
          </div>
        </div>

        {#if task.description}
          <p class="line-clamp-2 text-xs text-muted-foreground">
            {stripMarkdown(task.description)}
          </p>
        {/if}

        {#if task.type || isOverdue(task) || isDueToday(task) || isDueThisWeek(task) || blocked || tags.length > 0}
          <div class="flex flex-wrap items-center gap-1">
            {#if task.type}
              <Badge variant="outline">{task.type}</Badge>
            {/if}
            {#if isOverdue(task)}
              <Badge variant="destructive"><FaCircleExclamation /> Overdue</Badge>
            {:else if isDueToday(task)}
              <Badge
                variant="secondary"
                class="bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300"
              >
                <FaClock /> Today
              </Badge>
            {:else if isDueThisWeek(task)}
              <Badge variant="outline"><FaCalendarWeek /> This week</Badge>
            {/if}
            {#if blocked}
              <Badge
                variant="outline"
                class="border-red-200 text-red-600 dark:border-red-800 dark:text-red-400"
              >
                <FaLock /> Blocked
              </Badge>
            {/if}
            {#each tags.slice(0, 2) as tag (tag)}
              <Badge variant="secondary" class="max-w-[8rem] truncate">{tag}</Badge>
            {/each}
            {#if tags.length > 2}
              <Badge variant="secondary" title={tags.slice(2).join(', ')}>
                +{tags.length - 2}
              </Badge>
            {/if}
          </div>
        {/if}

        <div class="flex items-center justify-between gap-2">
          <div
            class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground"
          >
            {#if project && onProjectClick}
              <Button
                variant="link"
                size="sm"
                class="flex h-auto min-w-0 items-center gap-1 p-0 text-xs text-muted-foreground hover:text-foreground"
                onclick={(e) => {
                  e.stopPropagation();
                  onProjectClick();
                }}
                title={project}
              >
                <FaFolder class="h-3 w-3 shrink-0" />
                <span class="max-w-[10rem] truncate">{project}</span>
              </Button>
            {:else if project}
              <span class="flex min-w-0 items-center gap-1" title={project}>
                <FaFolder class="h-3 w-3 shrink-0" />
                <span class="max-w-[10rem] truncate">{project}</span>
              </span>
            {/if}
            {#if subCount > 0}
              <span class="flex items-center gap-1" title="{subCount} subtasks">
                <FaTurnDown class="h-3 w-3" />{subCount}
              </span>
            {/if}
            {#if task.assignee}
              <span class="flex items-center gap-1">
                <FaUser class="h-3 w-3" />
                <span class="max-w-[8rem] truncate">{task.assignee}</span>
              </span>
            {/if}
            {#if task.estimatedTime}
              <span class="flex items-center gap-1">
                <FaClock class="h-3 w-3" />{formatMinutes(task.estimatedTime)}
              </span>
            {/if}
          </div>

          <div class="flex shrink-0 items-center gap-1">
            {#if showSubtaskActions && !task.parentId}
              <Button
                onclick={(e) => {
                  e.stopPropagation();
                  onCreateSubtask();
                }}
                variant="ghost"
                size="sm"
                class="h-8 w-8 p-0"
                title="Create subtask"
                aria-label="Create subtask"
              >
                <FaPlus class="h-4 w-4 text-muted-foreground" />
              </Button>
            {/if}
            {#if !readonly}
              <Button
                onclick={(e) => {
                  e.stopPropagation();
                  onToggleStatus();
                }}
                variant="ghost"
                size="sm"
                class="h-8 w-8 p-0"
                aria-label="Toggle task status"
                title="Toggle task status"
              >
                <Icon
                  icon={display.icon(task)}
                  class="h-4 w-4 {display.statusColor(task.status)}"
                />
              </Button>
            {/if}
          </div>
        </div>
      </CardContent>
    </Card>
  {/snippet}
</ContextMenuList>
