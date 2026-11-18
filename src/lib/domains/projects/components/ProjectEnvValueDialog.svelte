<!-- Edit one variable's value in one .env file. -->
<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    envKey: string;
    fileName: string;
    value: string;
    onSubmit: (value: string) => Promise<void>;
    onClose: () => void;
  }

  const { envKey, fileName, value, onSubmit, onClose }: Props = $props();

  let draft = $state(value);
  let busy = $state(false);

  async function submit() {
    busy = true;
    try {
      await onSubmit(draft);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog open onOpenChange={(o) => !o && onClose()}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle class="font-mono text-sm">{envKey}</DialogTitle>
      <p class="text-xs text-muted-foreground">Writes to {fileName}</p>
    </DialogHeader>
    <Textarea bind:value={draft} rows={3} class="font-mono" />
    <DialogFooter>
      <Button variant="ghost" onclick={onClose}>Cancel</Button>
      <Button onclick={submit} disabled={busy}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
