<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import type { GeneratedTaskStructure } from '../services/aiTaskService';
  import type { TaskStatus } from '../types';
  import TaskPreviewCard from './TaskPreviewCard.svelte';
  import TaskPreviewList from './TaskPreviewList.svelte';

  interface Props {
    data: GeneratedTaskStructure;
    originalStatus?: TaskStatus;
    isUpdateMode: boolean;
    onUpdateMain: (task: any) => void;
    onUpdateSubtask: (index: number, subtask: any) => void;
    onAdd: (task: any) => Promise<void>;
    onAddSubtask: (index: number, subtask: any) => void;
    onAddAll: () => void;
  }
  let {
    data = $bindable(),
    originalStatus,
    isUpdateMode,
    onUpdateMain,
    onUpdateSubtask,
    onAdd,
    onAddSubtask,
    onAddAll,
  }: Props = $props();
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-xl font-bold">Preview</h2>
      <p class="text-sm text-muted-foreground">
        Review and edit the generated tasks before creating them
      </p>
    </div>
    <!-- 'existing-task' = loaded from a saved task, nothing generated yet -->
    {#if data.model_used !== 'existing-task'}
      <Badge variant="outline" class="text-xs">
        Confidence: {Math.round(data.confidence * 100)}% | Model:{' '}
        {data.model_used}
      </Badge>
    {/if}
  </div>

  <TaskPreviewCard
    task={data.main_task}
    bind:suggestedLabels={data.suggested_labels}
    onUpdate={onUpdateMain}
    {onAdd}
    {onAddAll}
    hasSubtasks={data.subtasks.length > 0}
    showAddButton={true}
    {originalStatus}
    {isUpdateMode}
  />

  <TaskPreviewList
    bind:subtasks={data.subtasks}
    onUpdate={onUpdateSubtask}
    onAdd={onAddSubtask}
    showAddButtons={true}
  />
</div>
