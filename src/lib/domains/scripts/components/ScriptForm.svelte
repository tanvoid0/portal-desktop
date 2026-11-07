<!--
	Script editor cards (basic information, command, parameters).
	Shared by /scripts/new and /scripts/[id]/edit — the page owns saving.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Badge } from '$lib/components/ui/badge';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import Select from '$lib/components/ui/select.svelte';
  import Trash2 from '~icons/fa6-solid/trash';
  import Plus from '~icons/fa6-solid/plus';
  import type { BlockParameter, CreateBlockRequest } from '$lib/domains/projects/pipelines';
  import { toast } from '$lib/utils/toast';

  interface Props {
    /** Bound, so a parent can replace the whole form at once (JSON import). */
    formData: CreateBlockRequest;
  }

  let { formData = $bindable() }: Props = $props();

  const commandPlaceholder = 'e.g., ./scripts/vpn.sh ${action} --config-dir ${configDir}';

  let showAddParameter = $state(false);
  let newParameter = $state<BlockParameter>({
    name: '',
    type: 'string',
    description: '',
    required: false,
    defaultValue: '',
  });

  function addParameter() {
    if (!newParameter.name.trim()) {
      toast.error('Parameter name is required');
      return;
    }
    if (formData.parameters.some((p) => p.name === newParameter.name)) {
      toast.error('Parameter name already exists');
      return;
    }
    formData.parameters = [...formData.parameters, { ...newParameter }];
    newParameter = {
      name: '',
      type: 'string',
      description: '',
      required: false,
      defaultValue: '',
    };
    showAddParameter = false;
  }

  function removeParameter(index: number) {
    formData.parameters = formData.parameters.filter((_, i) => i !== index);
  }
</script>

<div class="space-y-6">
  <!-- Basic Information -->
  <Card>
    <CardHeader>
      <CardTitle>Basic Information</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label for="name">Name *</Label>
          <Input
            id="name"
            bind:value={formData.name}
            placeholder="e.g., VPN Connect"
            class="mt-1"
          />
        </div>
        <div>
          <Label for="category">Category *</Label>
          <Select
            options={[
              { value: 'utility', label: 'Utility' },
              { value: 'build', label: 'Build' },
              { value: 'test', label: 'Test' },
              { value: 'deploy', label: 'Deploy' },
              { value: 'custom', label: 'Custom' },
            ]}
            bind:value={formData.category}
            placeholder="Select category"
            class="mt-1"
          />
        </div>
      </div>
      <div>
        <Label for="description">Description *</Label>
        <Textarea
          id="description"
          bind:value={formData.description}
          placeholder="Describe what this script does..."
          rows={3}
          class="mt-1"
        />
      </div>
      <div>
        <Label for="tags">Tags</Label>
        <Input
          id="tags"
          value={(formData.tags || []).join(', ')}
          oninput={(e) => {
            formData.tags = (e.target as HTMLInputElement).value
              .split(',')
              .map((t) => t.trim())
              .filter((t) => t.length > 0);
          }}
          placeholder="e.g., vpn, network, security (comma-separated)"
          class="mt-1"
        />
      </div>
    </CardContent>
  </Card>

  <!-- Command Configuration -->
  <Card>
    <CardHeader>
      <CardTitle>Command Configuration</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <div>
        <Label for="command">Command / Script Path *</Label>
        <Textarea
          id="command"
          bind:value={formData.command}
          placeholder={commandPlaceholder}
          rows={3}
          class="mt-1 font-mono text-sm"
        />
        <p class="mt-2 text-xs text-muted-foreground">
          Use <code class="rounded bg-muted px-1">${'{paramName}'}</code> for parameter placeholders.
          These will be replaced with actual values when the script runs.
        </p>
      </div>
      <div>
        <Label for="executionType">Execution Type</Label>
        <Select
          options={[
            { value: 'script', label: 'Script' },
            { value: 'command', label: 'Command' },
            { value: 'docker', label: 'Docker' },
          ]}
          bind:value={formData.executionType}
          placeholder="Select type"
          class="mt-1"
        />
      </div>
    </CardContent>
  </Card>

  <!-- Parameters -->
  <Card>
    <CardHeader>
      <div class="flex items-center justify-between">
        <CardTitle>Parameters</CardTitle>
        <Button variant="outline" onclick={() => (showAddParameter = !showAddParameter)}>
          <Plus class="mr-2 h-4 w-4" />
          Add Parameter
        </Button>
      </div>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if showAddParameter}
        <div class="space-y-4 rounded-lg border bg-muted/30 p-4">
          <h4 class="font-medium">New Parameter</h4>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <Label class="text-sm">Name *</Label>
              <Input bind:value={newParameter.name} placeholder="e.g., configDir" class="mt-1" />
            </div>
            <div>
              <Label class="text-sm">Type</Label>
              <Select
                options={[
                  { value: 'string', label: 'String' },
                  { value: 'number', label: 'Number' },
                  { value: 'boolean', label: 'Boolean' },
                  { value: 'select', label: 'Select' },
                  { value: 'file', label: 'File' },
                  { value: 'directory', label: 'Directory' },
                ]}
                bind:value={newParameter.type}
                class="mt-1"
              />
            </div>
          </div>
          <div>
            <Label class="text-sm">Description</Label>
            <Input
              bind:value={newParameter.description}
              placeholder="Describe this parameter..."
              class="mt-1"
            />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <Label class="text-sm">Default Value</Label>
              <Input
                bind:value={newParameter.defaultValue}
                placeholder="Default value (optional)"
                class="mt-1"
              />
            </div>
            <div class="flex items-end gap-2 pb-2">
              <Checkbox bind:checked={newParameter.required} id="new-param-required" />
              <Label for="new-param-required" class="text-sm">Required parameter</Label>
            </div>
          </div>
          <div class="flex gap-2">
            <Button onclick={addParameter}>Add Parameter</Button>
            <Button variant="outline" onclick={() => (showAddParameter = false)}>Cancel</Button>
          </div>
        </div>
      {/if}

      {#if formData.parameters.length > 0}
        <div class="divide-y rounded-lg border">
          {#each formData.parameters as param, i}
            <div class="flex items-center justify-between p-4">
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <code class="rounded bg-muted px-2 py-0.5 font-mono text-sm font-medium">
                    ${'{' + param.name + '}'}
                  </code>
                  <Badge variant="outline">{param.type}</Badge>
                  {#if param.required}
                    <Badge variant="destructive" class="text-xs">required</Badge>
                  {/if}
                </div>
                {#if param.description}
                  <p class="text-sm text-muted-foreground">
                    {param.description}
                  </p>
                {/if}
                {#if param.defaultValue !== undefined && param.defaultValue !== ''}
                  <p class="text-xs text-muted-foreground">
                    Default: <code class="rounded bg-muted px-1">{param.defaultValue}</code>
                  </p>
                {/if}
              </div>
              <Button
                variant="ghost"
                size="icon"
                aria-label={`Remove parameter ${param.name}`}
                onclick={() => removeParameter(i)}
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
          {/each}
        </div>
      {:else}
        <div class="py-8 text-center text-muted-foreground">
          <p>No parameters defined yet.</p>
          <p class="text-sm">Parameters allow users to customize script behavior at runtime.</p>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
