<script lang="ts">
  import FaCircleExclamation from '~icons/fa6-solid/circle-exclamation';
  import FaClock from '~icons/fa6-solid/clock';
  import FaCalendarWeek from '~icons/fa6-solid/calendar-week';
  import FaStopwatch from '~icons/fa6-solid/stopwatch';
  import FaLock from '~icons/fa6-solid/lock';
  import FaLockOpen from '~icons/fa6-solid/lock-open';
  import FaTag from '~icons/fa6-solid/tag';
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaComment from '~icons/fa6-solid/comment';
  import FaPaperclip from '~icons/fa6-solid/paperclip';
  import FaPlus from '~icons/fa6-solid/plus';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import { taskUi, getSubtaskCount } from '../state/taskUi.svelte';
  import {
    getTaskStatusColor,
    getPriorityColor,
    getTaskIcon,
    isOverdue,
    isDueToday,
    isDueThisWeek,
    formatMinutes,
  } from '../utils/taskDisplay';
  import type { Task } from '../types';

  interface Props {
    task: Task;
    onClick?: () => void;
    onStatusToggle?: () => void;
    onCreateSubtask?: () => void;
    showCheckbox?: boolean;
    isSelected?: boolean;
    onSelectionChange?: () => void;
  }

  let {
    task,
    onClick,
    onStatusToggle,
    onCreateSubtask,
    showCheckbox = false,
    isSelected = false,
    onSelectionChange,
  }: Props = $props();

  function isCurrentlyTracking(task: Task): boolean {
    return taskUi.currentlyTracking?.taskId === task.id;
  }

  function isBlocked(task: Task): boolean {
    return !!task.blockedBy && task.blockedBy.length > 0;
  }

  function isBlocking(task: Task): boolean {
    return !!task.blocks && task.blocks.length > 0;
  }
</script>

<Card
  class="cursor-pointer transition-colors {isSelected
    ? 'bg-warning-50 ring-2 ring-warning-500 dark:bg-warning-900/20'
    : ''}"
  elevation={isSelected ? 'elevated' : 'raised'}
  borderAccent={task.priority === 'high' ? 'left' : 'none'}
  onclick={onClick}
>
  <CardContent class="p-4">
    <div class="mb-2 flex items-start justify-between">
      {#if showCheckbox}
        <Checkbox
          checked={isSelected}
          onCheckedChange={() => onSelectionChange?.()}
          onclick={(e) => e.stopPropagation()}
          aria-label="Select task"
          class="mt-1"
        />
      {/if}
      <div class="flex-1">
        <div class="mb-1 flex items-center gap-2">
          <h4 class="text-sm font-medium text-foreground">{task.title}</h4>

          <!-- Due Date Indicators -->
          {#if isOverdue(task)}
            <Badge variant="destructive" class="text-xs">
              <FaCircleExclamation class="mr-1 h-3 w-3" />
              Overdue
            </Badge>
          {:else if isDueToday(task)}
            <Badge
              variant="secondary"
              class="bg-yellow-100 text-xs text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300"
            >
              <FaClock class="mr-1 h-3 w-3" />
              Due Today
            </Badge>
          {:else if isDueThisWeek(task)}
            <Badge variant="outline" class="text-xs">
              <FaCalendarWeek class="mr-1 h-3 w-3" />
              This Week
            </Badge>
          {/if}

          <!-- Time Tracking Indicator -->
          {#if isCurrentlyTracking(task)}
            <Badge
              variant="secondary"
              class="bg-blue-100 text-xs text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
            >
              <FaStopwatch class="mr-1 h-3 w-3" />
              Tracking
            </Badge>
          {/if}

          <!-- Dependency Indicators -->
          {#if isBlocked(task)}
            <Badge
              variant="outline"
              class="border-red-200 text-xs text-red-600 dark:border-red-800 dark:text-red-400"
            >
              <FaLock class="mr-1 h-3 w-3" />
              Blocked
            </Badge>
          {/if}

          {#if isBlocking(task)}
            <Badge
              variant="outline"
              class="border-orange-200 text-xs text-orange-600 dark:border-orange-800 dark:text-orange-400"
            >
              <FaLockOpen class="mr-1 h-3 w-3" />
              Blocking
            </Badge>
          {/if}
        </div>

        <!-- Tags -->
        {#if task.tags && task.tags.length > 0}
          <div class="mb-2 flex flex-wrap gap-1">
            {#each task.tags as tag}
              <Badge variant="outline" class="text-xs">
                <FaTag class="mr-1 h-3 w-3" />
                {tag}
              </Badge>
            {/each}
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-1">
        {#if getSubtaskCount(task.id, taskUi.tasks) > 0}
          <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <FaTurnDown class="h-3 w-3" />
            <span>{getSubtaskCount(task.id, taskUi.tasks)}</span>
          </div>
        {/if}
        <FaFlag class="h-3 w-3 {getPriorityColor(task.priority)}" />
        <span class="text-xs text-muted-foreground uppercase">{task.priority}</span>
      </div>
    </div>

    {#if task.description}
      <p class="mb-3 line-clamp-2 text-xs text-muted-foreground">
        {task.description}
      </p>
    {/if}

    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        {#if task.type}
          <Badge variant="outline" class="text-xs">
            {task.type}
          </Badge>
        {/if}

        <!-- Time Estimates -->
        {#if task.estimatedTime || task.actualTime}
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            {#if task.estimatedTime}
              <div class="flex items-center gap-1">
                <FaClock class="h-3 w-3" />
                <span>{formatMinutes(task.estimatedTime)}</span>
              </div>
            {/if}
            {#if task.actualTime}
              <div class="flex items-center gap-1">
                <FaClock class="h-3 w-3" />
                <span>{formatMinutes(task.actualTime)}</span>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Comments and Attachments Count -->
        {#if task.comments && task.comments.length > 0}
          <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <FaComment class="h-3 w-3" />
            <span>{task.comments.length}</span>
          </div>
        {/if}

        {#if task.attachments && task.attachments.length > 0}
          <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <FaPaperclip class="h-3 w-3" />
            <span>{task.attachments.length}</span>
          </div>
        {/if}
      </div>

      <div class="flex items-center gap-1">
        {#if onCreateSubtask && !task.parentId}
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
        <Button
          onclick={(e) => {
            e.stopPropagation();
            onStatusToggle?.();
          }}
          variant="ghost"
          size="sm"
          class="h-8 w-8 p-0"
          aria-label="Toggle task status"
          title="Toggle task status"
        >
          <Icon icon={getTaskIcon(task)} class="h-4 w-4 {getTaskStatusColor(task.status)}" />
        </Button>
      </div>
    </div>
  </CardContent>
</Card>
