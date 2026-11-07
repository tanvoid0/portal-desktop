<!--
	Create New Script Page
	Dedicated page for creating new scripts with JSON import support
-->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { PageHeader } from '$lib/components/shell';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import Upload from '~icons/fa6-solid/upload';
  import Save from '~icons/fa6-solid/floppy-disk';
  import { parseImportJson } from '$lib/domains/automation/utils/importJson';
  import type { CreateBlockRequest } from '$lib/domains/projects/pipelines';
  import { blockLibraryService } from '$lib/domains/projects/pipelines';
  import { ScriptForm } from '$lib/domains/scripts';
  import { toast } from '$lib/utils/toast';
  import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';

  // Set breadcrumb
  setBreadcrumbs([
    { label: 'Scripts', href: '/scripts' },
    { label: 'New Script', href: '/scripts/new' },
  ]);

  // Check if we should auto-open import section from URL query
  onMount(() => {
    const importParam = page.url.searchParams.get('import');
    if (importParam === 'true') {
      showJsonImport = true;
    }
  });

  // Form state
  let formData = $state<CreateBlockRequest>({
    name: '',
    description: '',
    category: 'utility',
    parameters: [],
    command: '',
    executionType: 'script',
    defaultConfig: {},
    tags: [],
  });

  // JSON import state
  let jsonImportText = $state('');
  let jsonImportError = $state('');
  let showJsonImport = $state(false);

  const jsonImportPlaceholder =
    '{"name": "My Script", "command": "./script.sh", "parameters": [], ...}';

  let saving = $state(false);

  function handleJsonImport() {
    jsonImportError = '';
    if (!jsonImportText.trim()) {
      jsonImportError = 'Please paste JSON content';
      return;
    }
    const result = parseImportJson(jsonImportText, 'script');
    if ('error' in result) {
      jsonImportError = result.error;
      return;
    }
    formData = result.data;
    jsonImportText = '';
    showJsonImport = false;
    toast.success('JSON imported - review and save');
  }

  function handleJsonFileImport(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      jsonImportText = e.target?.result as string;
      handleJsonImport();
    };
    reader.readAsText(file);
    target.value = '';
  }

  async function handleSave() {
    if (saving) return;
    if (!formData.name || !formData.description || !formData.command) {
      toast.error('Please fill in all required fields');
      return;
    }

    saving = true;
    try {
      // Build defaultConfig from parameters with defaultValue
      const defaultConfig: Record<string, unknown> = {};
      formData.parameters.forEach((p) => {
        if (p.defaultValue !== undefined && p.defaultValue !== '') {
          defaultConfig[p.name] = p.defaultValue;
        }
      });
      formData.defaultConfig = defaultConfig;

      await blockLibraryService.createBlock(formData);
      toast.success('Script created successfully');
      goto('/scripts');
    } catch (error) {
      console.error('Failed to create script', error);
      toast.error('Failed to create script');
    } finally {
      saving = false;
    }
  }

  const isValid = $derived(formData.name && formData.description && formData.command);
</script>

<svelte:head>
  <title>New Script - Portal Desktop</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
  <PageHeader title="New script">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={() => (showJsonImport = true)}>
        <Upload class="mr-2 h-4 w-4" />
        Import JSON
      </Button>
      <Button variant="outline" size="sm" onclick={() => goto('/scripts')}>Cancel</Button>
      <Button size="sm" onclick={handleSave} disabled={!isValid || saving}>
        <Save class="mr-2 h-4 w-4" />
        {saving ? 'Saving...' : 'Save Script'}
      </Button>
    {/snippet}
  </PageHeader>

  <Dialog bind:open={showJsonImport}>
    <DialogContent class="max-w-xl">
      <DialogHeader>
        <DialogTitle>Import from JSON</DialogTitle>
        <DialogDescription>Quickly populate fields from a JSON file or paste</DialogDescription>
      </DialogHeader>
      <div class="space-y-4">
        <div class="flex gap-2">
          <input
            type="file"
            id="json-file-import"
            accept=".json"
            class="hidden"
            onchange={handleJsonFileImport}
          />
          <Button
            variant="outline"
            onclick={() => document.getElementById('json-file-import')?.click()}
          >
            <Upload class="mr-2 h-4 w-4" />
            Load from File
          </Button>
        </div>
        <div>
          <Label>Or paste JSON directly</Label>
          <Textarea
            bind:value={jsonImportText}
            placeholder={jsonImportPlaceholder}
            rows={6}
            class="mt-2 font-mono text-sm"
          />
        </div>
        {#if jsonImportError}
          <p class="text-sm text-destructive">{jsonImportError}</p>
        {/if}
      </div>
      <DialogFooter>
        <Button onclick={handleJsonImport} disabled={!jsonImportText.trim()}>
          Import & Preview
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <div class="space-y-6">
    <ScriptForm bind:formData />

    <!-- Preview -->
    {#if formData.name || formData.command}
      <Card>
        <CardHeader>
          <CardTitle>Preview</CardTitle>
        </CardHeader>
        <CardContent>
          <pre
            class="max-h-60 overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">{JSON.stringify(
              formData,
              null,
              2
            )}</pre>
        </CardContent>
      </Card>
    {/if}
  </div>
</div>
