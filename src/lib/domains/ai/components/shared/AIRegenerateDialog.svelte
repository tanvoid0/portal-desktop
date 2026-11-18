<!-- Dialog for regenerating a previous AI result with extra instructions. -->
<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import FaRotate from '~icons/fa6-solid/rotate';

  interface Props {
    open: boolean;
    preview: string;
    instructions: string;
    error: string | null;
    isRegenerating: boolean;
    onCancel: () => void;
    onRegenerate: () => void;
  }

  let {
    open = $bindable(),
    preview,
    instructions = $bindable(),
    error,
    isRegenerating,
    onCancel,
    onRegenerate,
  }: Props = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[90vh] max-w-2xl overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Regenerate with Instructions</Dialog.Title>
      <Dialog.Description>
        Provide instructions on how to modify the previous generation. The AI will use the previous
        result and your instructions to create an improved version.
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label>Previous Result (Reference)</Label>
        <div class="max-h-[300px] overflow-y-auto rounded-lg border bg-muted/50 p-4">
          <pre class="font-mono text-xs whitespace-pre-wrap">{preview}</pre>
        </div>
      </div>

      <div class="space-y-2">
        <Label for="regenerate-instructions">Instructions for Regeneration</Label>
        <Textarea
          id="regenerate-instructions"
          bind:value={instructions}
          placeholder="e.g., Make the titles more concise, add 2 more subtasks, change priority to high..."
          rows={6}
          class={error ? 'border-destructive' : ''}
          disabled={isRegenerating}
        />
        <p class="text-xs text-muted-foreground">
          Describe what changes you'd like to make to the previous generation.
        </p>
        {#if error}
          <p class="text-sm text-destructive">{error}</p>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={onCancel} disabled={isRegenerating}>Cancel</Button>
      <Button onclick={onRegenerate} disabled={isRegenerating || !instructions.trim()}>
        <FaRotate class="mr-2 h-4 w-4" />
        Regenerate
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
