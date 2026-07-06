<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import Icon from "@iconify/svelte";
  import { documentActions } from "$lib/domains/documents";
  import DocumentEditor from "$lib/domains/documents/components/DocumentEditor.svelte";
  import type { Document } from "$lib/domains/documents";
  import { PageLoading, PageError } from "$lib/components/shell";
  import { toastActions } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";

  let document = $state<Document | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  const documentId = $derived(parseInt($page.params.id));

  onMount(async () => {
    await loadDocument();
  });

  async function loadDocument() {
    isLoading = true;
    error = null;
    try {
      const doc = await documentActions.getDocument(documentId);
      if (doc) {
        document = doc;
      } else {
        error = "Document not found";
      }
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load document";
      toastActions.error("Failed to load document", error);
    } finally {
      isLoading = false;
    }
  }

  async function handleSave(savedDoc: Document) {
    document = savedDoc;
    toastActions.success("Document saved successfully");
  }

  async function handleDelete() {
    if (!document) return;
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${document.title}"?`,
      "Delete document",
    );
    if (!confirmed) return;

    try {
      await documentActions.deleteDocument(document.id);
      toastActions.success("Document deleted successfully");
      goto("/documents");
    } catch (err) {
      toastActions.error(
        "Failed to delete document",
        err instanceof Error ? err.message : "An unexpected error occurred",
      );
    }
  }
</script>

<svelte:head>
  <title>{document ? document.title : "Document"} - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
  <div class="mb-4">
    <Button variant="ghost" onclick={() => goto("/documents")}>
      <Icon icon="lucide:arrow-left" class="mr-2 h-4 w-4" />
      Back to Documents
    </Button>
  </div>

  {#if isLoading}
    <PageLoading message="Loading document..." />
  {:else if error}
    <PageError
      title="Failed to load document"
      message={error}
      onRetry={loadDocument}
    />
  {:else if document}
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold">{document.title}</h1>
        <Button variant="destructive" onclick={handleDelete}>
          <Icon icon="lucide:trash-2" class="mr-2 h-4 w-4" />
          Delete
        </Button>
      </div>
      <DocumentEditor {document} onSave={handleSave} />
    </div>
  {/if}
</div>
