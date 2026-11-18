<!-- Dialog to create a GitHub issue in one of the user's repositories. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Dialog from '$lib/components/ui/dialog';

  interface Props {
    open: boolean;
    repoOptions: { value: string; label: string }[];
    reposPending: boolean;
    repoFullName: string;
    title: string;
    body: string;
    submitting: boolean;
    onCreate: () => void;
  }

  let {
    open = $bindable(),
    repoOptions,
    reposPending,
    repoFullName = $bindable(),
    title = $bindable(),
    body = $bindable(),
    submitting,
    onCreate,
  }: Props = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>Create Issue</Dialog.Title>
    </Dialog.Header>
    <div class="space-y-4">
      <Select
        bind:value={repoFullName}
        options={repoOptions}
        placeholder={reposPending ? 'Loading repositories...' : 'Select a repository'}
      />
      <Input bind:value={title} placeholder="Issue title" />
      <Textarea bind:value={body} placeholder="Issue description" />
      <Button onclick={onCreate} disabled={submitting || !repoFullName || !title.trim()}>
        Create Issue
      </Button>
    </div>
  </Dialog.Content>
</Dialog.Root>
