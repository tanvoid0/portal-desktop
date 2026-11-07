<!--
	Import a block from a JSON file. Hands the raw file text to the caller.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import Upload from '~icons/fa6-solid/upload';
  import { toast } from '$lib/utils/toast';

  interface Props {
    open?: boolean;
    onImport?: (json: string) => void;
  }

  let { open = $bindable(false), onImport = () => {} }: Props = $props();

  let fileInput = $state<HTMLInputElement | null>(null);

  async function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    target.value = '';
    if (!file) return;

    if (!file.name.endsWith('.json')) {
      toast.error('YAML import not yet supported. Please use JSON format.');
      return;
    }

    onImport(await file.text());
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Import Block</Dialog.Title>
      <Dialog.Description>Import a block from a JSON file</Dialog.Description>
    </Dialog.Header>
    <p class="text-sm text-muted-foreground">
      Select a JSON file containing block definition. The file should match the CreateBlockRequest
      format.
    </p>
    <input
      type="file"
      accept=".json"
      class="hidden"
      bind:this={fileInput}
      onchange={handleFileSelect}
    />
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={() => fileInput?.click()}>
        <Upload class="mr-2 h-4 w-4" />
        Select File
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
