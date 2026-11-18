<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent } from '$lib/components/ui/card';

  interface Props {
    name: string;
    instruction: string;
    editing: boolean;
    onSubmit: () => void;
    onCancel: () => void;
  }
  let {
    name = $bindable(),
    instruction = $bindable(),
    editing,
    onSubmit,
    onCancel,
  }: Props = $props();
</script>

<Card>
  <CardContent class="space-y-3 pt-4">
    <div class="space-y-2">
      <Label for="template-name">Template Name</Label>
      <Input id="template-name" bind:value={name} placeholder="e.g., API Development" />
    </div>
    <div class="space-y-2">
      <Label for="template-instruction">Instruction</Label>
      <Textarea
        id="template-instruction"
        bind:value={instruction}
        placeholder="Enter the instruction text..."
        rows={3}
      />
    </div>
    <div class="flex gap-2">
      <Button size="sm" onclick={onSubmit}>
        {editing ? 'Update' : 'Create'} Template
      </Button>
      {#if editing}
        <Button size="sm" variant="outline" onclick={onCancel}>Cancel</Button>
      {/if}
    </div>
  </CardContent>
</Card>
