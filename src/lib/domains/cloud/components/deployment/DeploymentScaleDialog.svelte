<!-- Dialog to scale a deployment's replica count -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';

  interface Props {
    open: boolean;
    name: string;
    replicas: number;
    scaling: boolean;
    onConfirm: () => void;
  }
  let { open = $bindable(), name, replicas = $bindable(), scaling, onConfirm }: Props = $props();
</script>

<Dialog bind:open>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Scale Deployment: {name}</DialogTitle>
      <DialogDescription>
        Enter the desired number of replicas for this deployment.
      </DialogDescription>
    </DialogHeader>
    <div class="grid gap-4 py-4">
      <div class="grid grid-cols-4 items-center gap-4">
        <Label for="replicas" class="text-right">Replicas</Label>
        <Input id="replicas" type="number" min="0" bind:value={replicas} class="col-span-3" />
      </div>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => (open = false)} disabled={scaling}>Cancel</Button>
      <Button onclick={onConfirm} disabled={scaling}>
        {scaling ? 'Scaling...' : 'Scale'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
