<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import type { BackendInfo } from '../generated';

  interface Props {
    backends: BackendInfo[];
    busy?: boolean;
    onStart: (backendId: string, cwd: string, prompt: string, model: string | null) => void;
  }

  let { backends, busy = false, onStart }: Props = $props();

  let backendId = $state('');
  let cwd = $state('');
  let model = $state('');
  let prompt = $state('');

  $effect(() => {
    if (!backendId && backends.length > 0) backendId = backends[0].id;
  });

  const backendOptions = $derived(backends.map((b) => ({ value: b.id, label: b.display_name })));
  const selected = $derived(backends.find((b) => b.id === backendId));
  const canStart = $derived(!busy && !!backendId && cwd.trim() !== '' && prompt.trim() !== '');

  async function browse() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') cwd = dir;
  }

  function submit() {
    if (!canStart) return;
    onStart(backendId, cwd.trim(), prompt.trim(), model.trim() || null);
  }
</script>

<div class="flex flex-col gap-3 p-4">
  <div class="grid gap-1.5">
    <Label for="agent-backend">Agent</Label>
    <Select
      id="agent-backend"
      options={backendOptions}
      value={backendId}
      onValueChange={(v) => (backendId = v)}
      placeholder="Select an agent"
    />
    {#if selected && !selected.available}
      <p class="text-xs text-destructive">
        {selected.display_name} wasn't found on PATH — install it to run this.
      </p>
    {/if}
  </div>

  <div class="grid gap-1.5">
    <Label for="agent-cwd">Working directory</Label>
    <div class="flex gap-2">
      <Input id="agent-cwd" bind:value={cwd} placeholder="Pick a folder…" class="flex-1" />
      <Button size="icon" variant="ghost" title="Browse for a folder" onclick={browse}>
        <FolderOpen class="h-4 w-4" />
      </Button>
    </div>
  </div>

  {#if selected && selected.models.length > 0}
    <div class="grid gap-1.5">
      <Label for="agent-model">Model</Label>
      <Select
        id="agent-model"
        options={selected.models.map((m) => ({ value: m.id, label: m.label }))}
        value={model}
        onValueChange={(v) => (model = v)}
        placeholder="Default"
      />
    </div>
  {/if}

  <div class="grid gap-1.5">
    <Label for="agent-prompt">First message</Label>
    <Textarea
      id="agent-prompt"
      bind:value={prompt}
      rows={4}
      placeholder="What should the agent do?"
    />
  </div>

  <Button disabled={!canStart} onclick={submit}>
    {busy ? 'Starting…' : 'Start run'}
  </Button>
</div>
