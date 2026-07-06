<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from "$lib/components/ui/dialog";
  import Icon from "@iconify/svelte";
  import { documentActions, documents } from "../stores/documentStore";
  import type { Document } from "../types";
  import { goto } from "$app/navigation";

  interface Props {
    selectedDocumentId?: number | null;
    onSelect?: (doc: Document | null) => void;
    onCreateNew?: () => void;
  }

  let { selectedDocumentId, onSelect, onCreateNew }: Props = $props();

  let isOpen = $state(false);
  let searchQuery = $state("");
  let selectedDoc = $derived(
    selectedDocumentId
      ? $documents.find((d) => d.id === selectedDocumentId)
      : null,
  );

  const filteredDocuments = $derived(
    searchQuery.trim()
      ? $documents.filter(
          (doc) =>
            !doc.isArchived &&
            (doc.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
              doc.content.toLowerCase().includes(searchQuery.toLowerCase())),
        )
      : $documents.filter((doc) => !doc.isArchived),
  );

  async function handleSelect(doc: Document) {
    onSelect?.(doc);
    isOpen = false;
  }

  function handleCreateNew() {
    if (onCreateNew) {
      onCreateNew();
    } else {
      goto("/documents/create");
    }
    isOpen = false;
  }

  function handleClear() {
    onSelect?.(null);
    isOpen = false;
  }

  // Load documents when dialog opens
  $effect(() => {
    if (isOpen) {
      documentActions.loadDocuments();
    }
  });
</script>

<div class="space-y-2">
  <Label>Linked Document</Label>
  {#if selectedDoc}
    <Card class="p-3">
      <div class="flex items-center justify-between">
        <div class="flex min-w-0 flex-1 items-center gap-2">
          <Icon
            icon="lucide:file-text"
            class="h-4 w-4 flex-shrink-0 text-muted-foreground"
          />
          <span class="truncate font-medium">{selectedDoc.title}</span>
          {#if selectedDoc.isDraft}
            <Badge variant="outline" class="text-xs">Draft</Badge>
          {/if}
        </div>
        <div class="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            onclick={() => goto(`/documents/${selectedDoc.id}`)}
            class="h-8"
          >
            <Icon icon="lucide:external-link" class="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm" onclick={handleClear} class="h-8">
            <Icon icon="lucide:x" class="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Card>
  {:else}
    <Dialog bind:open={isOpen}>
      <DialogTrigger>
        <Button variant="outline" class="w-full justify-start">
          <Icon icon="lucide:link" class="mr-2 h-4 w-4" />
          Link Document
        </Button>
      </DialogTrigger>
      <DialogContent class="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Select or Create Document</DialogTitle>
        </DialogHeader>
        <div class="space-y-4">
          <div class="flex gap-2">
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
            <Button onclick={handleCreateNew}>
              <Icon icon="lucide:plus" class="mr-2 h-4 w-4" />
              New Document
            </Button>
          </div>
          <div class="max-h-[400px] space-y-2 overflow-y-auto">
            {#if filteredDocuments.length === 0}
              <div class="py-8 text-center text-muted-foreground">
                <Icon
                  icon="lucide:file-x"
                  class="mx-auto mb-2 h-8 w-8 opacity-50"
                />
                <p class="text-sm">No documents found</p>
              </div>
            {:else}
              {#each filteredDocuments as doc}
                <Card
                  class="cursor-pointer transition-colors hover:bg-muted"
                  onclick={() => handleSelect(doc)}
                >
                  <CardContent class="p-3">
                    <div class="flex items-center justify-between">
                      <div class="min-w-0 flex-1">
                        <p class="truncate font-medium">{doc.title}</p>
                        <p class="line-clamp-1 text-sm text-muted-foreground">
                          {doc.content.substring(0, 100)}
                          {doc.content.length > 100 ? "..." : ""}
                        </p>
                      </div>
                      <Icon
                        icon="lucide:chevron-right"
                        class="ml-2 h-4 w-4 text-muted-foreground"
                      />
                    </div>
                  </CardContent>
                </Card>
              {/each}
            {/if}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  {/if}
</div>
