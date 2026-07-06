<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import Icon from "@iconify/svelte";
  import { FileText } from "@lucide/svelte";
  import { PageEmpty } from "$lib/components/shell";
  import DocumentCard from "./DocumentCard.svelte";
  import type { Document } from "../types";

  interface Props {
    documents: Document[];
    onDocumentClick?: (doc: Document) => void;
    onDocumentDelete?: (doc: Document) => void;
    onCreateNew?: () => void;
  }

  let { documents, onDocumentClick, onDocumentDelete, onCreateNew }: Props =
    $props();

  let searchQuery = $state("");
  const filteredDocuments = $derived(
    searchQuery.trim()
      ? documents.filter(
          (doc) =>
            doc.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
            doc.content.toLowerCase().includes(searchQuery.toLowerCase()),
        )
      : documents,
  );
</script>

<div class="space-y-4">
  <div class="flex items-center gap-2">
    <div class="relative flex-1">
      <Icon
        icon="lucide:search"
        class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        type="text"
        placeholder="Search documents..."
        bind:value={searchQuery}
        class="pl-10"
      />
    </div>
    {#if onCreateNew}
      <Button onclick={onCreateNew}>
        <Icon icon="lucide:plus" class="mr-2 h-4 w-4" />
        New Document
      </Button>
    {/if}
  </div>

  {#if filteredDocuments.length === 0}
    <PageEmpty
      title="No documents found"
      description="Create your first document to get started."
      filteredDescription="No documents match your search."
      isFiltered={Boolean(searchQuery.trim())}
      icon={FileText}
      actionLabel="New Document"
      onAction={onCreateNew}
    />
  {:else}
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredDocuments as doc}
        <DocumentCard
          document={doc}
          onClick={() => onDocumentClick?.(doc)}
          onDelete={() => onDocumentDelete?.(doc)}
        />
      {/each}
    </div>
  {/if}
</div>
