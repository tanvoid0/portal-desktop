<!-- Dialog to edit, close or reopen a GitHub issue. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Dialog from '$lib/components/ui/dialog';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';

  interface Props {
    open: boolean;
    title: string;
    body: string;
    htmlUrl?: string | null;
    submitting: boolean;
    onUpdate: (nextState?: 'open' | 'closed') => void;
  }

  let {
    open = $bindable(),
    title = $bindable(),
    body = $bindable(),
    htmlUrl,
    submitting,
    onUpdate,
  }: Props = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>Edit Issue</Dialog.Title>
    </Dialog.Header>
    <div class="space-y-4">
      <Input bind:value={title} placeholder="Issue title" />
      <Textarea bind:value={body} placeholder="Issue description" />
      {#if htmlUrl}
        <a
          href={htmlUrl}
          target="_blank"
          rel="noreferrer"
          class="inline-flex items-center text-sm text-primary"
        >
          Open on GitHub
          <ExternalLink class="ml-1 h-4 w-4" />
        </a>
      {/if}
      <div class="flex flex-wrap gap-2">
        <Button onclick={() => onUpdate(undefined)} disabled={submitting}>Save</Button>
        <Button variant="outline" onclick={() => onUpdate('closed')} disabled={submitting}>
          Close
        </Button>
        <Button variant="outline" onclick={() => onUpdate('open')} disabled={submitting}>
          Reopen
        </Button>
      </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
