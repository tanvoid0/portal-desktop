<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import DocumentEditor from "$lib/domains/documents/components/DocumentEditor.svelte";
  import type { Document } from "$lib/domains/documents";
  import { toastActions } from "$lib/utils/toast";
  import { taskActions } from "$lib/domains/tasks";
  import { fetchTaskById } from "$lib/domains/tasks/api/taskApi";
  import { ResourceType } from "$lib/domains/shared/types/resourceType";

  let taskId = $derived($page.url.searchParams.get("taskId"));
  let taskTitle = $derived($page.url.searchParams.get("taskTitle"));
  let taskDescription = $state<string | undefined>(undefined);

  $effect(() => {
    if (!taskId) {
      taskDescription = undefined;
      return;
    }

    void fetchTaskById(taskId)
      .then((task) => {
        taskDescription = task?.description;
      })
      .catch((err) => {
        console.error("Failed to load task:", err);
      });
  });

  async function handleSave(savedDoc: Document) {
    toastActions.success("Document created successfully");

    // If taskId is provided, link the document to the task
    if (taskId) {
      try {
        await taskActions.updateTask(taskId, {
          resourceId: savedDoc.id.toString(),
          resourceType: ResourceType.DOCUMENT,
        });
        toastActions.success("Document linked to task");
        goto(`/tasks/${taskId}`);
        return;
      } catch (err) {
        console.error("Failed to link document to task:", err);
        toastActions.error(
          "Failed to link document to task",
          err instanceof Error ? err.message : "Unknown error",
        );
      }
    }

    goto(`/documents/${savedDoc.id}`);
  }

  function handleCancel() {
    if (taskId) {
      goto(`/tasks/${taskId}`);
    } else {
      goto("/documents");
    }
  }
</script>

<svelte:head>
  <title>Create Document - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
  <DocumentEditor
    onSave={handleSave}
    onCancel={handleCancel}
    initialTitle={taskTitle || undefined}
    initialContent={taskTitle ? `Document for task: ${taskTitle}` : undefined}
  />
</div>
