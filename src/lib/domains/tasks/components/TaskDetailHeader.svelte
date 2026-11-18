<script lang="ts">
  import FaArrowLeft from '~icons/fa6-solid/arrow-left';
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import FaPencil from '~icons/fa6-solid/pencil';
  import FaTrash from '~icons/fa6-solid/trash';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import Icon from '@iconify/svelte';
  import {
    getTaskStatusColor,
    getStatusBadgeColor,
    getTaskIcon,
    activeSession,
  } from '../utils/taskDisplay';
  import { taskStatusLabel, type Task } from '../types';

  interface Props {
    task: Task;
    /** Leading button; hidden when not supplied. */
    onBack?: () => void;
    isEditing: boolean;
    isSaving: boolean;
    onEdit: () => void;
    onCancel: () => void;
    onSave: () => void;
    onDelete: () => void;
    onResumeSession: (sessionId: string) => void;
  }
  let {
    task,
    onBack,
    isEditing,
    isSaving,
    onEdit,
    onCancel,
    onSave,
    onDelete,
    onResumeSession,
  }: Props = $props();
</script>

<div class="mb-6 flex flex-wrap items-center justify-between gap-2">
  <div class="flex items-center gap-4">
    {#if onBack}
      <Button variant="ghost" onclick={onBack} class="flex items-center gap-2">
        <FaArrowLeft class="h-4 w-4" />
        Back to Tasks
      </Button>
      <Separator orientation="vertical" class="h-6" />
    {/if}
    <div class="flex items-center gap-2">
      <Icon icon={getTaskIcon(task)} class="h-5 w-5 {getTaskStatusColor(task.status)}" />
      <Badge class={getStatusBadgeColor(task.status)}>
        {taskStatusLabel(task.status)}
      </Badge>
      {#if activeSession(task)}
        <Badge
          variant="outline"
          class="cursor-pointer gap-1.5 text-xs hover:bg-accent"
          title="Resume this Claude Code session in a terminal"
          onclick={() => onResumeSession(activeSession(task)!)}
        >
          <span
            class="h-2 w-2 shrink-0 animate-pulse rounded-full bg-emerald-500 shadow-[0_0_6px_theme(colors.emerald.500)]"
          ></span>
          <span class="max-w-[8ch] truncate text-muted-foreground">{activeSession(task)}</span>
        </Badge>
      {/if}
    </div>
  </div>

  <div class="flex flex-wrap items-center gap-2">
    {#if isEditing}
      <Button variant="outline" onclick={onCancel} disabled={isSaving}>Cancel</Button>
      <Button onclick={onSave} disabled={isSaving}>Save Changes</Button>
    {:else}
      <Button
        variant="outline"
        onclick={() => goto(`/tasks/${task.id}/generate`)}
        class="flex items-center gap-2"
      >
        <FaWandMagicSparkles class="h-4 w-4" />
        Generate Tasks with AI
      </Button>
      <Button variant="outline" onclick={onEdit} class="flex items-center gap-2">
        <FaPencil class="h-4 w-4" />
        Edit
      </Button>
      <Button variant="destructive" onclick={onDelete} class="flex items-center gap-2">
        <FaTrash class="h-4 w-4" />
        Delete
      </Button>
    {/if}
  </div>
</div>
