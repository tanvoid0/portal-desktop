<!--
	Global Blocks Management Page
	Frame for the Blocks list: one control row,
	data loading, and the dialogs.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Plus from '~icons/fa6-solid/plus';
  import Upload from '~icons/fa6-solid/upload';
  import type { Block, CreateBlockRequest } from '$lib/domains/projects/pipelines';
  import {
    blockLibraryService,
    blockLibraryStore,
    blocks as blocksStore,
  } from '$lib/domains/projects/pipelines';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import BlockList from './BlockList.svelte';
  import BlockEditDialog from './BlockEditDialog.svelte';
  import BlockImportDialog from './BlockImportDialog.svelte';

  let blocks = $state<Block[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let searchQuery = $state('');
  let selectedCategory = $state<Block['category'] | null>(null);

  let editingBlock = $state<Block | null>(null);
  let showBlockDialog = $state(false);
  let showImportDialog = $state(false);

  const categoryFilters: { value: Block['category'] | null; label: string }[] = [
    { value: null, label: 'All' },
    { value: 'build', label: 'Build' },
    { value: 'test', label: 'Test' },
    { value: 'deploy', label: 'Deploy' },
    { value: 'utility', label: 'Utility' },
  ];

  function clearFilters() {
    searchQuery = '';
    selectedCategory = null;
  }

  onMount(async () => {
    await loadBlocks();
  });

  async function loadBlocks() {
    loading = true;
    loadError = null;
    try {
      await blockLibraryStore.loadBlocks();
    } catch (error) {
      console.error('Failed to load blocks', error);
      loadError = error instanceof Error ? error.message : 'Failed to load blocks';
      toast.error('Failed to load blocks');
    } finally {
      loading = false;
    }
  }

  // Subscribe to blocks store
  $effect(() => {
    const unsubscribe = blocksStore.subscribe((b) => {
      blocks = b;
    });
    return unsubscribe;
  });

  function handleCreateBlock() {
    editingBlock = null;
    showBlockDialog = true;
  }

  function handleEditBlock(block: Block) {
    editingBlock = block;
    showBlockDialog = true;
  }

  async function handleSaveBlock(formData: CreateBlockRequest) {
    try {
      if (editingBlock) {
        await blockLibraryService.updateBlock(editingBlock.id, formData);
        toast.success('Block updated successfully');
      } else {
        await blockLibraryService.createBlock(formData);
        toast.success('Block created successfully');
      }
      showBlockDialog = false;
      await loadBlocks();
    } catch (error) {
      console.error('Failed to save block', error);
      toast.error('Failed to save block');
    }
  }

  async function handleDeleteBlock(blockId: string) {
    const confirmed = await confirmAction(
      'Are you sure you want to delete this block?',
      'Delete block',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;
    try {
      await blockLibraryService.deleteBlock(blockId);
      toast.success('Block deleted successfully');
      await loadBlocks();
    } catch (error) {
      console.error('Failed to delete block', error);
      toast.error('Failed to delete block');
    }
  }

  async function handleImportBlock(json: string) {
    try {
      await blockLibraryService.createBlock(JSON.parse(json) as CreateBlockRequest);
      toast.success('Block imported successfully');
      showImportDialog = false;
      await loadBlocks();
    } catch (error) {
      console.error('Failed to import block', error);
      toast.error('Failed to import block');
    }
  }

  function handleExportBlock(block: Block) {
    try {
      const filename = `${block.name.toLowerCase().replace(/\s+/g, '-')}.json`;
      downloadJson(JSON.stringify(block, null, 2), filename);
      toast.success('Block exported successfully');
    } catch (error) {
      console.error('Failed to export block', error);
      toast.error('Failed to export block');
    }
  }

  function downloadJson(json: string, filename: string) {
    const url = URL.createObjectURL(new Blob([json], { type: 'application/json' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="flex flex-col gap-4 pt-4">
  <div class="flex flex-wrap items-center gap-2">
    <Input
      bind:value={searchQuery}
      class="min-w-48 flex-1"
      aria-label="Search blocks"
      placeholder="Search blocks..."
    />
    <div class="flex flex-wrap items-center gap-1">
      {#each categoryFilters as filter (filter.label)}
        <Button
          variant={selectedCategory === filter.value ? 'default' : 'outline'}
          size="sm"
          onclick={() => (selectedCategory = filter.value)}
        >
          {filter.label}
        </Button>
      {/each}
    </div>
    <Button variant="outline" size="sm" onclick={() => (showImportDialog = true)}>
      <Upload class="mr-2 h-4 w-4" />
      Import
    </Button>
    <Button size="sm" onclick={handleCreateBlock}>
      <Plus class="mr-2 h-4 w-4" />
      Create
    </Button>
  </div>

  <BlockList
    {blocks}
    search={searchQuery}
    category={selectedCategory}
    {loading}
    error={loadError}
    onRetry={loadBlocks}
    onCreate={handleCreateBlock}
    onClearFilters={clearFilters}
    onEdit={handleEditBlock}
    onExport={handleExportBlock}
    onDelete={handleDeleteBlock}
  />
</div>

<BlockEditDialog bind:open={showBlockDialog} block={editingBlock} onSave={handleSaveBlock} />
<BlockImportDialog bind:open={showImportDialog} onImport={handleImportBlock} />
