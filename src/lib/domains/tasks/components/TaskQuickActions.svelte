<script lang="ts">
  import FaHourglassHalf from '~icons/fa6-solid/hourglass-half';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaPencil from '~icons/fa6-solid/pencil';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { taskActions } from '../stores/taskStore';
  import type { Task } from '../types';

  interface Props {
    task: Task;
    /** Bound to SubtaskList's inline add form. */
    showAddSubtask?: boolean;
  }

  let { task, showAddSubtask = $bindable(false) }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle class="text-lg">Quick Actions</CardTitle>
  </CardHeader>
  <CardContent class="space-y-2">
    <Button
      variant="outline"
      onclick={() => taskActions.toggleTaskStatus(task.id)}
      class="w-full justify-start"
    >
      <FaHourglassHalf class="mr-2 h-4 w-4" />
      Toggle Status
    </Button>

    {#if !task.parentId}
      <Button
        variant="outline"
        onclick={() => (showAddSubtask = !showAddSubtask)}
        class="w-full justify-start"
      >
        <FaPlus class="mr-2 h-4 w-4" />
        {showAddSubtask ? 'Cancel Add Subtask' : 'Add Subtask'}
      </Button>
    {/if}

    <Button
      variant="outline"
      onclick={() => goto(`/tasks/${task.id}/edit`)}
      class="w-full justify-start"
    >
      <FaPencil class="mr-2 h-4 w-4" />
      Edit Task
    </Button>
  </CardContent>
</Card>
