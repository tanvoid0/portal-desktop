<!--
	Create / edit a pipeline block. `block === null` is create mode, which also
	offers the paste-or-load JSON shortcut.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Badge } from '$lib/components/ui/badge';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import Upload from '~icons/fa6-solid/upload';
  import Trash2 from '~icons/fa6-solid/trash';
  import type { Block, CreateBlockRequest } from '$lib/domains/projects/pipelines';
  import { parseImportJson } from '$lib/domains/automation/utils/importJson';
  import { toast } from '$lib/utils/toast';

  interface Props {
    open?: boolean;
    block: Block | null;
    onSave?: (data: CreateBlockRequest) => void;
  }

  let { open = $bindable(false), block, onSave = () => {} }: Props = $props();

  const emptyForm = (): CreateBlockRequest => ({
    name: '',
    description: '',
    category: 'utility',
    parameters: [],
    command: '',
    executionType: 'command',
    defaultConfig: {},
    tags: [],
  });

  let formData = $state<CreateBlockRequest>(emptyForm());
  let jsonImportText = $state('');
  let jsonImportError = $state('');
  let jsonFileInput = $state<HTMLInputElement | null>(null);

  // Reload the form each time the dialog opens, from the block being edited.
  $effect(() => {
    if (!open) return;
    formData = block
      ? {
          name: block.name,
          description: block.description,
          category: block.category,
          parameters: block.parameters,
          command: block.command,
          executionType: block.executionType,
          defaultConfig: block.defaultConfig,
          tags: block.tags || [],
        }
      : emptyForm();
    jsonImportText = '';
    jsonImportError = '';
  });

  function handleJsonImport() {
    jsonImportError = '';
    if (!jsonImportText.trim()) {
      jsonImportError = 'Please paste JSON content';
      return;
    }
    const result = parseImportJson(jsonImportText, 'command');
    if ('error' in result) {
      jsonImportError = result.error;
      return;
    }
    formData = result.data;
    jsonImportText = '';
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
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90vh] max-w-3xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>{block ? 'Edit Block' : 'Create New Block'}</Dialog.Title>
      <Dialog.Description>
        {block
          ? 'Update block details'
          : 'Define a reusable block that can be used in pipeline steps'}
      </Dialog.Description>
    </Dialog.Header>
    <div class="space-y-4">
      <!-- JSON Import Section -->
      {#if !block}
        <div class="space-y-3 rounded-lg border border-dashed p-4">
          <div class="flex items-center justify-between">
            <Label class="text-sm font-medium">Import from JSON</Label>
            <input
              type="file"
              accept=".json"
              class="hidden"
              bind:this={jsonFileInput}
              onchange={handleJsonFileImport}
            />
            <Button variant="outline" size="sm" onclick={() => jsonFileInput?.click()}>
              <Upload class="mr-1 h-3 w-3" />
              Load File
            </Button>
          </div>
          <Textarea
            bind:value={jsonImportText}
            placeholder={'Paste JSON here, e.g.: {"name": "My Block", "command": "echo hello", ...}'}
            rows={3}
            class="font-mono text-xs"
          />
          {#if jsonImportError}
            <p class="text-sm text-destructive">{jsonImportError}</p>
          {/if}
          <Button
            variant="secondary"
            size="sm"
            onclick={handleJsonImport}
            disabled={!jsonImportText.trim()}
          >
            Import & Preview
          </Button>
        </div>
      {/if}

      <!-- Form Fields -->
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label for="block-name">Name *</Label>
          <Input
            id="block-name"
            bind:value={formData.name}
            placeholder="e.g., Install Dependencies"
          />
        </div>
        <div>
          <Label for="block-category">Category *</Label>
          <Select
            id="block-category"
            options={[
              { value: 'build', label: 'Build' },
              { value: 'test', label: 'Test' },
              { value: 'deploy', label: 'Deploy' },
              { value: 'utility', label: 'Utility' },
              { value: 'custom', label: 'Custom' },
            ]}
            bind:value={formData.category}
            placeholder="Select category"
          />
        </div>
      </div>
      <div>
        <Label for="block-description">Description *</Label>
        <Textarea
          id="block-description"
          bind:value={formData.description}
          placeholder="Describe what this block does..."
          rows={3}
        />
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label for="block-execution-type">Execution Type *</Label>
          <Select
            id="block-execution-type"
            options={[
              { value: 'command', label: 'Command' },
              { value: 'script', label: 'Script' },
              { value: 'docker', label: 'Docker' },
            ]}
            bind:value={formData.executionType}
            placeholder="Select execution type"
          />
        </div>
        <div>
          <Label for="block-command">Command *</Label>
          <Input id="block-command" bind:value={formData.command} placeholder="e.g., npm install" />
        </div>
      </div>
      <div>
        <Label for="block-tags">Tags (comma-separated)</Label>
        <Input
          id="block-tags"
          value={(formData.tags || []).join(', ')}
          oninput={(e) => {
            formData.tags = (e.target as HTMLInputElement).value
              .split(',')
              .map((t) => t.trim())
              .filter((t) => t.length > 0);
          }}
          placeholder="e.g., node, npm, install"
        />
      </div>

      <!-- Parameters Section -->
      {#if formData.parameters && formData.parameters.length > 0}
        <div class="space-y-2">
          <Label>Parameters ({formData.parameters.length})</Label>
          <div class="max-h-40 space-y-2 overflow-y-auto rounded-md border p-3">
            {#each formData.parameters as param, i}
              <div class="flex items-center justify-between rounded bg-muted/50 p-2 text-sm">
                <div>
                  <span class="font-medium">{param.name}</span>
                  <span class="ml-2 text-muted-foreground">({param.type})</span>
                  {#if param.required}
                    <Badge variant="outline" class="ml-2 text-xs">required</Badge>
                  {/if}
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={`Remove parameter ${param.name}`}
                  onclick={() => {
                    formData.parameters = formData.parameters.filter((_, idx) => idx !== i);
                  }}
                >
                  <Trash2 class="h-3 w-3" />
                </Button>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Default Config Preview -->
      {#if formData.defaultConfig && Object.keys(formData.defaultConfig).length > 0}
        <div class="space-y-2">
          <Label>Default Config</Label>
          <pre class="overflow-x-auto rounded-md bg-muted p-3 text-xs">{JSON.stringify(
              formData.defaultConfig,
              null,
              2
            )}</pre>
        </div>
      {/if}
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button
        onclick={() => onSave(formData)}
        disabled={!formData.name || !formData.description || !formData.command}
      >
        {block ? 'Update' : 'Create'}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
