<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import ProjectCombobox from '$lib/domains/projects/components/ProjectCombobox.svelte';
  import type { BackendInfo } from '../generated';

  const CWD_STORAGE_KEY = 'portal-agents-last-cwd';

  interface Props {
    backends: BackendInfo[];
    busy?: boolean;
    /** Pre-fills the working directory, e.g. a "New run" link from a project's
     *  session group. Seeded before `ProjectCombobox` mounts, so its own
     *  default-value effect (which only runs when `cwd` is still empty) no-ops. */
    initialCwd?: string;
    /** Pre-fills the first message, e.g. a task handed off from another page. */
    initialPrompt?: string;
    onStart: (backendId: string, cwd: string, prompt: string, model: string | null) => void;
  }

  let { backends, busy = false, initialCwd, initialPrompt, onStart }: Props = $props();

  let backendId = $state('');
  let cwd = $state(initialCwd ?? '');
  let model = $state('');
  let prompt = $state(initialPrompt ?? '');

  $effect(() => {
    if (!backendId && backends.length > 0) backendId = backends[0].id;
  });

  const backendOptions = $derived(backends.map((b) => ({ value: b.id, label: b.display_name })));
  const selected = $derived(backends.find((b) => b.id === backendId));
  const canStart = $derived(!busy && !!backendId && cwd.trim() !== '' && prompt.trim() !== '');

  function submit() {
    if (!canStart) return;
    onStart(backendId, cwd.trim(), prompt.trim(), model.trim() || null);
  }
</script>

<div class="mx-auto flex w-full max-w-xl flex-col gap-3 p-4">
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
    <Label>Working directory</Label>
    <ProjectCombobox
      bind:value={cwd}
      placeholder="Select a project…"
      storageKey={CWD_STORAGE_KEY}
    />
  </div>

  {#if selected}
    <div class="grid gap-1.5">
      <Label for="agent-model">Model</Label>
      {#if selected.models.length > 0}
        <Select
          id="agent-model"
          options={selected.models.map((m) => ({ value: m.id, label: m.label }))}
          value={model}
          onValueChange={(v) => (model = v)}
          placeholder="Default"
        />
      {:else}
        <!-- This backend has no enumerable model list (see `BackendModel`) —
             free-text so the user can still pick a model instead of being
             stuck on the backend's default. -->
        <Input id="agent-model" bind:value={model} placeholder="Default (type a model name)" />
      {/if}
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
