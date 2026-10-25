<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import { toast } from '$lib/utils/toast';
  import { Sparkles, LoaderCircle } from '@lucide/svelte';

  let {
    projectId,
    yaml = $bindable(),
    error = $bindable(null),
    aiAvailable,
    onvalidate,
  }: {
    projectId: number;
    yaml: string;
    error: string | null;
    aiAvailable: () => Promise<boolean>;
    onvalidate: () => void;
  } = $props();

  let refineOpen = $state(false);
  let ai = $state<boolean | null>(null);
  let instruction = $state('');
  let refining = $state(false);

  $effect(() => {
    if (refineOpen && ai === null) aiAvailable().then((ok) => (ai = ok));
  });

  async function refine() {
    refining = true;
    try {
      yaml = await invoke<string>('release_refine', {
        projectId,
        yaml,
        instruction: instruction.trim(),
      });
      error = null;
      instruction = '';
    } catch (e) {
      toast.error(String(e));
    } finally {
      refining = false;
    }
  }
</script>

<div class="space-y-3">
  <Textarea
    bind:value={yaml}
    onblur={onvalidate}
    rows={10}
    spellcheck={false}
    class="font-mono text-xs"
  />
  {#if error}<p class="text-xs text-destructive">{error}</p>{/if}

  <Collapsible bind:open={refineOpen}>
    <CollapsibleTrigger>
      {#snippet child({ props })}
        <Button {...props} variant="ghost" size="sm" class="text-muted-foreground">
          <Sparkles class="mr-2 h-4 w-4" /> Refine with AI
        </Button>
      {/snippet}
    </CollapsibleTrigger>
    <CollapsibleContent class="space-y-2 pt-2">
      {#if ai === null}
        <p class="text-xs text-muted-foreground">Checking AI…</p>
      {:else if ai}
        <Textarea
          bind:value={instruction}
          rows={2}
          placeholder="e.g. also publish the docker image to ghcr.io"
        />
        <Button
          size="sm"
          variant="outline"
          disabled={refining || !instruction.trim()}
          onclick={refine}
        >
          {#if refining}<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />{/if}
          {refining ? 'Refining…' : 'Refine'}
        </Button>
      {:else}
        <p class="text-xs text-muted-foreground">AI is not configured: Settings > AI</p>
      {/if}
    </CollapsibleContent>
  </Collapsible>
</div>
