<script lang="ts">
  import FaFlag from '~icons/fa6-solid/flag';
  import FaCalendar from '~icons/fa6-solid/calendar';
  import FaHourglassHalf from '~icons/fa6-solid/hourglass-half';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import MarkdownView from '$lib/components/ui/markdown-view.svelte';
  import {
    getTaskStatusColor,
    getStatusBadgeColor,
    getPriorityColor,
    getTaskIcon,
    formatMinutes,
  } from '../utils/taskDisplay';
  import { taskStatusLabel, type Task } from '../types';
  import TaskDetailField from './TaskDetailField.svelte';

  interface Props {
    task: Task;
  }
  let { task }: Props = $props();
</script>

<div class="space-y-4">
  <div>
    <h2 class="text-2xl font-bold text-foreground">
      {task.title}
    </h2>
    {#if task.description}
      <div class="mt-3">
        <MarkdownView content={task.description} truncateAt={500} />
      </div>
    {:else}
      <p class="mt-2 text-muted-foreground italic">No description provided</p>
    {/if}
  </div>

  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
    <TaskDetailField label="Status">
      <div class="mt-1 flex items-center gap-2">
        <Icon icon={getTaskIcon(task)} class="h-4 w-4 {getTaskStatusColor(task.status)}" />
        <Badge class={getStatusBadgeColor(task.status)}>
          {taskStatusLabel(task.status)}
        </Badge>
      </div>
    </TaskDetailField>

    <TaskDetailField label="Priority">
      <div class="mt-1 flex items-center gap-2">
        <FaFlag class="h-4 w-4 {getPriorityColor(task.priority)}" />
        <Badge variant="outline" class="uppercase">
          {task.priority}
        </Badge>
      </div>
    </TaskDetailField>
  </div>

  {#if task.type}
    <TaskDetailField label="Type">
      <div class="mt-1">
        <Badge variant="outline">{task.type}</Badge>
      </div>
    </TaskDetailField>
  {/if}

  {#if task.dueDate}
    <TaskDetailField label="Due Date">
      <div class="mt-1 flex items-center gap-2">
        <FaCalendar class="h-4 w-4 text-muted-foreground" />
        <span class="text-sm">{new Date(task.dueDate).toLocaleDateString()}</span>
      </div>
    </TaskDetailField>
  {/if}

  {#if task.assignee}
    <TaskDetailField label="Assignee">
      <div class="mt-1 text-sm">{task.assignee}</div>
    </TaskDetailField>
  {/if}

  {#if task.estimatedTime || task.actualTime}
    <TaskDetailField label="Time">
      <div class="mt-1 flex items-center gap-2 text-sm">
        <FaHourglassHalf class="h-4 w-4 text-muted-foreground" />
        <span>
          {task.estimatedTime ? formatMinutes(task.estimatedTime) : '-'} estimated{task.actualTime
            ? ` / ${formatMinutes(task.actualTime)} spent`
            : ''}
        </span>
      </div>
    </TaskDetailField>
  {/if}

  {#if task.tags && task.tags.length > 0}
    <TaskDetailField label="Tags">
      <div class="mt-1 flex flex-wrap gap-1">
        {#each task.tags as tag, i (i)}
          <Badge variant="outline">{tag}</Badge>
        {/each}
      </div>
    </TaskDetailField>
  {/if}
</div>
