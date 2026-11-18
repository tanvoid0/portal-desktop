<!-- Add/edit dialog for a settings item (name, category, icon, icon type). -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Select from '$lib/components/ui/select.svelte';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';

  interface Props {
    open: boolean;
    itemLabel: string;
    editing: boolean;
    name: string;
    category: string;
    icon: string;
    iconType: 'devicon' | 'file';
    onSave: () => void;
  }

  let {
    open = $bindable(),
    itemLabel,
    editing,
    name = $bindable(),
    category = $bindable(),
    icon = $bindable(),
    iconType = $bindable(),
    onSave,
  }: Props = $props();
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-[500px]">
    <DialogHeader>
      <DialogTitle>
        {editing ? `Edit ${itemLabel}` : `Add ${itemLabel}`}
      </DialogTitle>
      <DialogDescription>
        {editing
          ? `Update ${itemLabel.toLowerCase()} details`
          : `Add a new ${itemLabel.toLowerCase()} to your collection`}
      </DialogDescription>
    </DialogHeader>

    <div class="grid gap-4 py-4">
      <div class="space-y-2">
        <Label for="item-name">{itemLabel} Name *</Label>
        <Input id="item-name" bind:value={name} placeholder="Enter name..." />
      </div>
      <div class="space-y-2">
        <Label for="item-category">Category</Label>
        <Input id="item-category" bind:value={category} placeholder="Enter category..." />
      </div>
      <div class="space-y-2">
        <Label for="item-icon">Icon</Label>
        <Input
          id="item-icon"
          bind:value={icon}
          placeholder="logos:example (for Devicon) or image URL"
        />
        <p class="text-xs text-muted-foreground">
          Enter a Devicon name (e.g., "logos:javascript") or an image URL
        </p>
      </div>
      <div class="space-y-2">
        <Label for="item-icon-type">Icon Type</Label>
        <Select
          bind:value={iconType}
          options={[
            { value: 'devicon', label: 'Devicon' },
            { value: 'file', label: 'Image URL' },
          ]}
        />
      </div>
    </div>

    <DialogFooter>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={onSave}>
        {editing ? 'Update' : 'Add'}
        {itemLabel}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
