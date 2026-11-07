<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Dialog from '$lib/components/ui/dialog';
  import { toast } from '$lib/utils/toast';
  import { ideService, type IdeConfig } from '$lib/domains/ide';

  interface Props {
    open?: boolean;
    /** IDE being edited; null when adding. */
    ide: IdeConfig | null;
    /** Initial field values (already path-normalized by the caller). */
    initialName: string;
    initialExecutable: string;
    onSaved?: () => void;
  }

  let {
    open = $bindable(false),
    ide,
    initialName,
    initialExecutable,
    onSaved = () => {},
  }: Props = $props();

  let name = $state('');
  let executable = $state('');

  // Runs before render so the fields are filled by the time the dialog content mounts.
  $effect.pre(() => {
    if (open) {
      name = initialName;
      executable = initialExecutable;
    }
  });

  async function save() {
    if (!name.trim() || !executable.trim()) {
      toast.error('IDE name and executable path are required');
      return;
    }

    try {
      if (ide && ide.id) {
        await ideService.updateIde(ide.id, name, executable);
        toast.success('IDE updated successfully');
      } else {
        await ideService.addIde(name, executable);
        toast.success('IDE added successfully');
      }
      onSaved();
      open = false;
    } catch (error: any) {
      toast.error('Failed to save IDE', error);
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-md">
    <Dialog.Header>
      <Dialog.Title>{ide ? 'Edit IDE' : 'Add IDE'}</Dialog.Title>
    </Dialog.Header>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="ide-name">IDE Name</Label>
        <Input id="ide-name" bind:value={name} placeholder="e.g., Visual Studio Code" required />
      </div>

      <div class="space-y-2">
        <Label for="ide-executable">Executable Path</Label>
        <Input
          id="ide-executable"
          bind:value={executable}
          placeholder="e.g., /usr/bin/code or C:\Program Files\Code\code.exe"
          required
        />
        <p class="text-xs text-muted-foreground">Full path to the IDE executable</p>
      </div>

      <Dialog.Footer>
        <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button onclick={save}>{ide ? 'Update' : 'Add'} IDE</Button>
      </Dialog.Footer>
    </div>
  </Dialog.Content>
</Dialog.Root>
