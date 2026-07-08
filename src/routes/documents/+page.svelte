<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import Icon from "@iconify/svelte";
  import {
    documentActions,
    documents,
    isLoading,
    error,
  } from "$lib/domains/documents";
  import DocumentList from "$lib/domains/documents/components/DocumentList.svelte";
  import type { Document } from "$lib/domains/documents";
  import {
    PageHeader,
    PageLoading,
    PageError,
  } from "$lib/components/shell";
  import { toastActions } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";

  onMount(() => {
    documentActions.loadDocuments();
  });

  async function handleDocumentClick(doc: Document) {
    goto(`/documents/${doc.id}`);
  }

  async function handleDocumentDelete(doc: Document) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${doc.title}"?`,
      "Delete document",
    );
    if (!confirmed) return;

    try {
      await documentActions.deleteDocument(doc.id);
      toastActions.success("Document deleted successfully");
    } catch (err) {
      toastActions.error(
        "Failed to delete document",
        err instanceof Error ? err.message : "An unexpected error occurred",
      );
    }
  }

  function handleCreateNew() {
    goto("/documents/create");
  }
</script>

<svelte:head>
  <title>Documents - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Documents"
    description="Workspace documentation, notes, and knowledge base"
  >
    {#snippet actions()}
      <Button variant="outline" onclick={() => goto("/documents/generate")}>
        <Icon icon="lucide:sparkles" class="mr-2 h-4 w-4" />
        Generate with AI
      </Button>
      <Button onclick={handleCreateNew}>
        <Icon icon="lucide:plus" class="mr-2 h-4 w-4" />
        New Document
      </Button>
    {/snippet}
  </PageHeader>

  {#if $isLoading}
    <PageLoading message="Loading documents..." />
  {:else if $error}
    <PageError
      title="Failed to load documents"
      message={$error}
      onRetry={() => documentActions.loadDocuments()}
    />
  {:else}
      <DocumentList
        documents={$documents.filter((d) => !d.isArchived)}
        onDocumentClick={handleDocumentClick}
        onDocumentDelete={handleDocumentDelete}
        onCreateNew={handleCreateNew}
      />
  {/if}
</div>
