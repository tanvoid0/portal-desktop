<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import TaskForm from "$lib/domains/tasks/components/TaskForm.svelte";
  import { createTaskQuery } from "$lib/domains/tasks/queries/taskQueries";
  import { toastActions } from "$lib/utils/toast";
  import { PageLoading, PageError } from "$lib/components/shell";
  import type { Task } from "$lib/domains/tasks/types";

  const taskId = $derived($page.params.id);
  const taskQuery = createTaskQuery(() => taskId);

  const task = $derived(taskQuery.data ?? null);
  const isLoading = $derived(taskQuery.isPending);
  const error = $derived(
    taskQuery.isError
      ? taskQuery.error instanceof Error
        ? taskQuery.error.message
        : "Failed to load task"
      : taskQuery.isSuccess && !taskQuery.data
        ? "Task not found"
        : null,
  );

  $effect(() => {
    if (taskQuery.isSuccess && !taskQuery.data) {
      toastActions.error(
        "Task not found",
        "The requested task could not be found",
      );
    }
  });

  function handleSave(updatedTask: Task) {
    goto(`/tasks/${updatedTask.id}`);
  }

  function handleCancel() {
    if (task) {
      goto(`/tasks/${task.id}`);
    } else {
      goto("/tasks");
    }
  }

  function reloadTask() {
    void taskQuery.refetch();
  }
</script>

<div class="container mx-auto p-6">
  {#if isLoading}
    <PageLoading message="Loading task..." />
  {:else if error}
    <PageError title="Failed to load task" message={error} onRetry={reloadTask} />
  {:else if task}
    <TaskForm {task} onSave={handleSave} onCancel={handleCancel} />
  {/if}
</div>
