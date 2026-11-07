<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import TaskDetail from '$lib/domains/tasks/components/TaskDetail.svelte';

  const taskId = $derived(page.params.id ?? '');
</script>

<div class="min-h-screen bg-background">
  <TaskDetail
    {taskId}
    onBack={() => goto('/tasks')}
    onDeleted={() => goto('/tasks')}
    onOpenTask={(id) => goto(`/tasks/${id}`)}
  />
</div>
