<!--
	Move a tool's big data folder (Ollama models, Gradle, Android emulators, caches)
	to another drive and repoint the tool at it. Read-only "Plan" first; "Move"
	only enables when nothing blocks it. Mirrors the `plan_relocation` /
	`apply_relocation` commands, which AI agents call too.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { RelocatableTool } from '../generated/RelocatableTool';
  import type { RelocationPlan } from '../generated/RelocationPlan';
  import type { RelocationResult } from '../generated/RelocationResult';
  import { fmtBytes } from '../utils';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

  let tools = $state<RelocatableTool[]>([]);
  let targetRoot = $state('');
  let plans = $state<Record<string, RelocationPlan>>({});
  let busyId = $state<string | null>(null);
  let error = $state('');

  const movable = $derived(tools.filter((t) => t.exists && !t.currentEnv));

  onMount(() => {
    void load();
  });

  async function load() {
    try {
      tools = await invoke<RelocatableTool[]>('list_relocatable_tools');
    } catch (e) {
      error = String(e);
    }
  }

  async function pickTarget() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') {
      targetRoot = dir;
      plans = {}; // plans are specific to a destination
    }
  }

  async function plan(tool: RelocatableTool) {
    if (!targetRoot) return toast.error('Pick a destination folder first');
    busyId = tool.id;
    try {
      plans[tool.id] = await invoke<RelocationPlan>('plan_relocation', {
        request: { toolId: tool.id, targetRoot },
      });
    } catch (e) {
      toast.error('Could not plan', String(e));
    } finally {
      busyId = null;
    }
  }

  async function move(tool: RelocatableTool, p: RelocationPlan) {
    const ok = await confirmAction(
      `Move ${fmtBytes(p.sourceBytes)} from ${p.source} to ${p.dest} and set ${p.envVar}?`,
      `Move ${tool.label}`
    );
    if (!ok) return;
    busyId = tool.id;
    try {
      const r = await invoke<RelocationResult>('apply_relocation', {
        request: { toolId: tool.id, targetRoot },
      });
      toast.success(`${tool.label} moved`, r.note);
      delete plans[tool.id];
      await load();
    } catch (e) {
      toast.error('Move failed', String(e));
    } finally {
      busyId = null;
    }
  }
</script>

<Card>
  <CardHeader>
    <CardTitle class="text-base">Move to another drive</CardTitle>
  </CardHeader>
  <CardContent class="space-y-3">
    <p class="text-sm text-muted-foreground">
      Moves a tool's data and points it at the new location with a user environment variable. Close
      the tool first; restart it afterwards.
    </p>

    <div class="flex flex-col gap-2 sm:flex-row">
      <Input bind:value={targetRoot} placeholder="Destination folder, e.g. I:\build" />
      <Button variant="outline" onclick={pickTarget}>Browse…</Button>
    </div>

    {#if error}
      <p class="text-sm text-destructive">{error}</p>
    {:else if movable.length === 0}
      <p class="text-sm text-muted-foreground">Nothing left on this drive to move.</p>
    {/if}

    {#each movable as t (t.id)}
      {@const p = plans[t.id]}
      <div class="rounded-md border p-3">
        <div class="flex items-center justify-between gap-2">
          <div class="min-w-0">
            <div class="text-sm font-medium">{t.label}</div>
            <div class="truncate text-xs text-muted-foreground">{t.source}</div>
          </div>
          <div class="flex shrink-0 gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={busyId !== null || !targetRoot}
              onclick={() => plan(t)}
            >
              Plan
            </Button>
            {#if p}
              <Button
                size="sm"
                disabled={busyId !== null || p.blockers.length > 0}
                onclick={() => move(t, p)}
              >
                Move {fmtBytes(p.sourceBytes)}
              </Button>
            {/if}
          </div>
        </div>
        {#if p}
          <ul class="mt-2 space-y-1 text-xs text-muted-foreground">
            {#each p.steps as s (s)}<li>{s}</li>{/each}
          </ul>
          {#each p.blockers as b (b)}
            <Badge variant="destructive" class="mt-2 mr-1">{b}</Badge>
          {/each}
        {/if}
      </div>
    {/each}
  </CardContent>
</Card>
