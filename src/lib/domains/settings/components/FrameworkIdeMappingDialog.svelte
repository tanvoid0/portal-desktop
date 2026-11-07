<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Dialog from '$lib/components/ui/dialog';
  import Select from '$lib/components/ui/select.svelte';
  import { toast } from '$lib/utils/toast';
  import { ideService, type IdeConfig, type FrameworkIdeMapping } from '$lib/domains/ide';

  interface Props {
    open?: boolean;
    /** Mapping being edited; null when creating. */
    mapping: FrameworkIdeMapping | null;
    /** Framework to pre-fill when creating (quick-add). */
    initialFramework?: string;
    ides: IdeConfig[];
    onSaved?: () => void;
  }

  let {
    open = $bindable(false),
    mapping,
    initialFramework = '',
    ides,
    onSaved = () => {},
  }: Props = $props();

  let framework = $state('');
  let ideId = $state('');

  // Runs before render so the fields are filled by the time the dialog content mounts.
  $effect.pre(() => {
    if (open) {
      framework = mapping?.framework ?? initialFramework;
      ideId = mapping ? String(mapping.ide_id) : '';
    }
  });

  async function save() {
    if (!framework.trim() || !ideId) {
      toast.error('Framework and IDE are required');
      return;
    }

    try {
      if (mapping) {
        // For updates, we need to delete and recreate since framework is the key
        await ideService.deleteFrameworkIdeMapping(mapping.framework);
      }
      await ideService.setFrameworkIdeMapping(framework, parseInt(ideId));
      toast.success(mapping ? 'Mapping updated successfully' : 'Mapping created successfully');
      onSaved();
      open = false;
    } catch (error: any) {
      toast.error('Failed to save mapping', error);
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-md">
    <Dialog.Header>
      <Dialog.Title>
        {mapping ? 'Edit Framework IDE Mapping' : 'Add Framework IDE Mapping'}
      </Dialog.Title>
    </Dialog.Header>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="mapping-framework">Framework</Label>
        <Input
          id="mapping-framework"
          bind:value={framework}
          placeholder="e.g., React, Vue, Angular, Node.js"
          required
        />
        <p class="text-xs text-muted-foreground">The framework name that will trigger this IDE</p>
      </div>

      <div class="space-y-2">
        <Label for="mapping-ide">IDE</Label>
        <Select
          defaultValue={ideId || ''}
          options={ides.map((ide) => ({
            value: String(ide.id || ''),
            label: ide.name,
          }))}
          onSelect={(value) => (ideId = value || '')}
          placeholder="Select an IDE..."
        />
        <p class="text-xs text-muted-foreground">The IDE that will open for this framework</p>
      </div>

      <Dialog.Footer>
        <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button onclick={save}>{mapping ? 'Update' : 'Create'} Mapping</Button>
      </Dialog.Footer>
    </div>
  </Dialog.Content>
</Dialog.Root>
