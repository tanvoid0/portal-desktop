<!--
	Edit a custom pipeline template's metadata. Steps and variables are
	export/import only.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import type { PipelineTemplate } from '$lib/domains/projects/pipelines';

  interface Props {
    open?: boolean;
    template: PipelineTemplate | null;
    onSave?: (data: Partial<PipelineTemplate>) => void;
  }

  let { open = $bindable(false), template, onSave = () => {} }: Props = $props();

  let formData = $state<Partial<PipelineTemplate>>({});

  $effect(() => {
    if (!open) return;
    formData = {
      name: template?.name ?? '',
      description: template?.description ?? '',
      framework: template?.framework ?? '',
      category: template?.category,
      tags: template?.tags || [],
    };
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90vh] max-w-2xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Edit Template</Dialog.Title>
      <Dialog.Description>
        Update template details. Note: Steps and variables cannot be edited here. Export and
        re-import to modify the full template structure.
      </Dialog.Description>
    </Dialog.Header>
    <div class="space-y-4">
      <div>
        <Label for="template-name">Name *</Label>
        <Input
          id="template-name"
          bind:value={formData.name}
          placeholder="e.g., Spring Boot Full Pipeline"
        />
      </div>
      <div>
        <Label for="template-description">Description *</Label>
        <Textarea
          id="template-description"
          bind:value={formData.description}
          placeholder="Describe what this template does..."
          rows={3}
        />
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label for="template-framework">Framework</Label>
          <Input
            id="template-framework"
            bind:value={formData.framework}
            placeholder="e.g., spring-boot, react"
          />
        </div>
        <div>
          <Label for="template-category">Category</Label>
          <Select
            id="template-category"
            options={[
              { value: 'build', label: 'Build' },
              { value: 'test', label: 'Test' },
              { value: 'deploy', label: 'Deploy' },
              { value: 'ci-cd', label: 'CI/CD' },
              { value: 'full-stack', label: 'Full Stack' },
            ]}
            bind:value={formData.category}
            placeholder="Select category"
          />
        </div>
      </div>
      <div>
        <Label for="template-tags">Tags (comma-separated)</Label>
        <Input
          id="template-tags"
          value={(formData.tags || []).join(', ')}
          oninput={(e) => {
            formData.tags = (e.target as HTMLInputElement).value
              .split(',')
              .map((t) => t.trim())
              .filter((t) => t.length > 0);
          }}
          placeholder="e.g., spring-boot, maven, docker"
        />
      </div>
      <p class="rounded-md bg-muted p-4 text-sm text-muted-foreground">
        <strong>Note:</strong> To edit steps, variables, or the full template structure, export the template,
        modify the JSON file, and import it back.
      </p>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={() => onSave(formData)} disabled={!formData.name || !formData.description}>
        Update Template
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
