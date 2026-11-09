<!--
	Global Blocks Management Page
	Frame for the Blocks / Templates lists: one control row, the tab switch,
	data loading, and the dialogs.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import Plus from '~icons/fa6-solid/plus';
  import Upload from '~icons/fa6-solid/upload';
  import type {
    Block,
    CreateBlockRequest,
    PipelineTemplate,
  } from '$lib/domains/projects/pipelines';
  import {
    blockLibraryService,
    blockLibraryStore,
    blocks as blocksStore,
  } from '$lib/domains/projects/pipelines';
  import { pipelineTemplateService } from '$lib/domains/projects/pipelines/services/pipelineTemplateService';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import BlockList from './BlockList.svelte';
  import BlockEditDialog from './BlockEditDialog.svelte';
  import BlockImportDialog from './BlockImportDialog.svelte';
  import TemplateList from './TemplateList.svelte';
  import TemplateEditDialog from './TemplateEditDialog.svelte';

  let blocks = $state<Block[]>([]);
  let templates = $state<PipelineTemplate[]>([]);
  let loading = $state(false);
  let loadError = $state<string | null>(null);
  let activeTab = $state<'blocks' | 'templates'>('blocks');
  let searchQuery = $state('');
  let selectedCategory = $state<Block['category'] | null>(null);

  let editingBlock = $state<Block | null>(null);
  let showBlockDialog = $state(false);
  let showImportDialog = $state(false);
  let editingTemplate = $state<PipelineTemplate | null>(null);
  let showTemplateDialog = $state(false);
  let templateFileInput = $state<HTMLInputElement | null>(null);

  const categoryFilters: { value: Block['category'] | null; label: string }[] = [
    { value: null, label: 'All' },
    { value: 'build', label: 'Build' },
    { value: 'test', label: 'Test' },
    { value: 'deploy', label: 'Deploy' },
    { value: 'utility', label: 'Utility' },
  ];

  // The search box is shared by both tabs, so both empty states clear both.
  function clearFilters() {
    searchQuery = '';
    selectedCategory = null;
  }

  onMount(async () => {
    await loadBlocks();
    loadTemplates();
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

  function loadTemplates() {
    templates = pipelineTemplateService.getAllTemplates();
  }

  function handleExportTemplate(template: PipelineTemplate) {
    try {
      downloadJson(pipelineTemplateService.exportTemplate(template.key), `${template.key}.json`);
      toast.success('Template exported successfully');
    } catch (error) {
      console.error('Failed to export template', error);
      toast.error('Failed to export template');
    }
  }

  async function handleTemplateFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    target.value = '';
    if (!file) return;

    try {
      pipelineTemplateService.importTemplate(await file.text());
      toast.success('Template imported successfully');
      loadTemplates();
    } catch (error) {
      console.error('Failed to import template', error);
      toast.error(error instanceof Error ? error.message : 'Failed to import template');
    }
  }

  async function handleDeleteTemplate(template: PipelineTemplate) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete template "${template.name}"?`,
      'Delete template',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;

    try {
      const deleted = pipelineTemplateService.deleteTemplate(template.key);
      if (!deleted) {
        toast.error('Failed to delete template');
        return;
      }
      toast.success('Template deleted successfully');
      loadTemplates();
    } catch (error) {
      console.error('Failed to delete template', error);
      toast.error(error instanceof Error ? error.message : 'Failed to delete template');
    }
  }

  function handleEditTemplate(template: PipelineTemplate) {
    editingTemplate = template;
    showTemplateDialog = true;
  }

  function handleSaveTemplate(formData: Partial<PipelineTemplate>) {
    if (!editingTemplate) return;
    try {
      pipelineTemplateService.updateTemplate(editingTemplate.key, formData);
      toast.success('Template updated successfully');
      showTemplateDialog = false;
      editingTemplate = null;
      loadTemplates();
    } catch (error) {
      console.error('Failed to update template', error);
      toast.error(error instanceof Error ? error.message : 'Failed to update template');
    }
  }
</script>

<!-- gap-4 (not space-y) because the Tabs root is a flex column. -->
<Tabs bind:value={activeTab} class="gap-4 pt-4">
  <!-- One control row: tabs, search, category filter, actions -->
  <div class="flex flex-wrap items-center gap-2">
    <TabsList>
      <TabsTrigger value="blocks">Blocks</TabsTrigger>
      <TabsTrigger value="templates">Templates</TabsTrigger>
    </TabsList>

    <Input
      bind:value={searchQuery}
      class="min-w-48 flex-1"
      aria-label={activeTab === 'blocks' ? 'Search blocks' : 'Search templates'}
      placeholder={activeTab === 'blocks' ? 'Search blocks...' : 'Search templates...'}
    />

    {#if activeTab === 'blocks'}
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
    {:else}
      <Button variant="outline" size="sm" onclick={() => templateFileInput?.click()}>
        <Upload class="mr-2 h-4 w-4" />
        Import Template
      </Button>
    {/if}
  </div>

  <TabsContent value="blocks">
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
  </TabsContent>

  <TabsContent value="templates">
    <TemplateList
      {templates}
      search={searchQuery}
      onImport={() => templateFileInput?.click()}
      onClearFilters={clearFilters}
      onEdit={handleEditTemplate}
      onExport={handleExportTemplate}
      onDelete={handleDeleteTemplate}
    />
  </TabsContent>
</Tabs>

<input
  type="file"
  accept=".json"
  class="hidden"
  bind:this={templateFileInput}
  onchange={handleTemplateFileSelect}
/>

<BlockEditDialog bind:open={showBlockDialog} block={editingBlock} onSave={handleSaveBlock} />
<BlockImportDialog bind:open={showImportDialog} onImport={handleImportBlock} />
<TemplateEditDialog
  bind:open={showTemplateDialog}
  template={editingTemplate}
  onSave={handleSaveTemplate}
/>
