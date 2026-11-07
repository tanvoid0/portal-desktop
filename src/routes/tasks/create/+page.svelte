<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import TaskForm from '$lib/domains/tasks/components/TaskForm.svelte';
  import { Button } from '$lib/components/ui/button';
  import { PageHeader } from '$lib/components/shell';
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import type { Task } from '$lib/domains/tasks/types';

  // Get parentId from URL params
  const parentId = page.url.searchParams.get('parentId') || undefined;

  function handleSave(task: Task) {
    // Navigate to task details or back to task list
    goto(`/tasks/${task.id}`);
  }

  function handleCancel() {
    goto('/tasks');
  }
</script>

<div class="space-y-6">
  <PageHeader title="New task">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={() => goto('/tasks/generate')}>
        <FaWandMagicSparkles class="mr-2 h-4 w-4" />
        Generate Tasks with AI
      </Button>
    {/snippet}
  </PageHeader>
  <TaskForm {parentId} onSave={handleSave} onCancel={handleCancel} />
</div>
