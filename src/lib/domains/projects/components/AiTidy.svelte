<!--
	AI tidy: ask the app's AI to propose a triage decision for every project that
	has none, then review and apply the ones you agree with. The AI only suggests;
	nothing is recorded until you apply, and a decision never touches the disk.
	Deletes are held back by code (see aiTidy.ts) when a project has work that
	exists nowhere else, whatever the model said.
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import Square from '~icons/fa6-solid/stop';
  import { invokeClient } from '$lib/utils/invokeClient';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { logger } from '$lib/domains/shared/services/logger';
  import { splitThinking } from '$lib/domains/ai/utils/thinking';
  import {
    setProjectTriageBulk,
    type TriageDecision,
  } from '$lib/domains/projects/api/discoveryApi';
  import {
    TIDY_SYSTEM,
    buildFacts,
    parseSuggestions,
    type TidyRow,
    type TidySuggestion,
  } from '$lib/domains/projects/utils/aiTidy';

  const log = logger.createScoped('AiTidy');

  /** Small enough that a local model answers in seconds and stays on task. */
  const BATCH = 8;
  const DESTRUCTIVE_CLASS = 'text-destructive hover:text-destructive hover:bg-destructive/10';

  type Item = TidySuggestion & { name: string };

  let running = $state(false);
  let stop = false;
  let done = $state(0);
  let total = $state(0);
  let skipped = $state(0);
  let error = $state<string | null>(null);
  let items = $state<Item[]>([]);
  let selected = $state<Set<number>>(new Set());
  let applying = $state(false);

  async function askBatch(rows: TidyRow[]): Promise<TidySuggestion[]> {
    const res = await invokeClient.post<{ content: string }>('generate_ai_text_with_system', {
      systemMessage: TIDY_SYSTEM,
      userMessage: buildFacts(rows),
      // Reasoning models spend most of the budget thinking; a small cap leaves
      // the answer empty. Measured: 400 tokens failed 23 of 24, 3000 passed all.
      options: { temperature: 0, max_tokens: 3000, timeout_ms: 180000 },
    });
    return parseSuggestions(splitThinking(res.content).answer, rows);
  }

  async function run() {
    running = true;
    stop = false;
    error = null;
    items = [];
    selected = new Set();
    done = 0;
    skipped = 0;
    try {
      const all = (await invokeClient.post<TidyRow[]>('get_all_projects')) ?? [];
      const todo = all.filter((r) => !r.triage_decision);
      total = todo.length;
      const names = new Map(todo.map((r) => [r.id, r.name]));
      for (let i = 0; i < todo.length && !stop; i += BATCH) {
        const batch = todo.slice(i, i + BATCH);
        try {
          const got = await askBatch(batch);
          skipped += batch.length - got.length;
          const fresh = got.map((s) => ({ ...s, name: names.get(s.id) ?? String(s.id) }));
          // Nothing is ticked for you: an unread suggestion should not be one click from applied.
          items = [...items, ...fresh];
        } catch (e) {
          // One bad batch (timeout, provider hiccup) should not discard the rest.
          log.warn('batch failed', e);
          skipped += batch.length;
          error = e instanceof Error ? e.message : String(e);
        }
        done = Math.min(i + BATCH, todo.length);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      running = false;
    }
  }

  // Review one kind of suggestion at a time instead of one 170-row wall.
  let filter = $state<'all' | TriageDecision>('all');
  const ORDER: TriageDecision[] = ['delete', 'review', 'archive', 'keep'];
  const counts = $derived.by(() => {
    const c: Record<string, number> = { all: items.length };
    for (const s of items) c[s.action] = (c[s.action] ?? 0) + 1;
    return c;
  });
  const shown = $derived(filter === 'all' ? items : items.filter((s) => s.action === filter));
  // A filter whose group was emptied (by applying it) falls back to everything.
  $effect(() => {
    if (filter !== 'all' && !counts[filter]) filter = 'all';
  });

  function selectShown() {
    selected = new Set([...selected, ...shown.map((s) => s.id)]);
  }

  function toggle(id: number) {
    const next = new Set(selected);
    if (!next.delete(id)) next.add(id);
    selected = next;
  }

  async function apply() {
    const chosen = items.filter((s) => selected.has(s.id));
    if (!chosen.length) return;
    const deletes = chosen.filter((s) => s.action === 'delete').length;
    if (deletes) {
      const ok = await confirmAction(
        'This only records the decision — nothing is removed from disk. Use the Disk tab to reclaim space.',
        `Mark ${deletes} project${deletes === 1 ? '' : 's'} for deletion?`
      );
      if (!ok) return;
    }
    applying = true;
    try {
      const groups = new Map<TriageDecision, number[]>();
      for (const s of chosen) groups.set(s.action, [...(groups.get(s.action) ?? []), s.id]);
      for (const [decision, ids] of groups) await setProjectTriageBulk(ids, decision);
      const applied = new Set(chosen.map((s) => s.id));
      items = items.filter((s) => !applied.has(s.id));
      selected = new Set();
      toast.success(`${chosen.length} decisions recorded`);
    } catch (e) {
      toast.error('Could not record the decisions', e);
    } finally {
      applying = false;
    }
  }

  const variant = (a: TriageDecision) =>
    a === 'delete' ? 'destructive' : a === 'keep' ? 'default' : 'secondary';
</script>

<Card>
  <CardHeader class="pb-3">
    <div class="flex items-center gap-2">
      <CardTitle class="flex items-center gap-2 text-base">
        <Sparkles class="h-4 w-4" /> AI tidy
      </CardTitle>
      {#if running}
        <Button size="sm" variant="outline" class="ml-auto" onclick={() => (stop = true)}>
          <Square class="mr-1 h-3 w-3" /> Stop
        </Button>
      {:else}
        <Button size="sm" class="ml-auto" onclick={run}>
          <Sparkles class="mr-1 h-3 w-3" />
          {items.length ? 'Ask again' : 'Ask AI'}
        </Button>
      {/if}
    </div>
    <p class="text-sm text-muted-foreground">
      The AI suggests keep, archive, delete or review for projects with no decision yet. You choose
      which to apply. A delete is held as “review” when a project has unpushed commits, stashes,
      uncommitted changes or no remote.
    </p>
  </CardHeader>
  <CardContent class="space-y-3">
    {#if running || done}
      <p class="text-sm text-muted-foreground tabular-nums" aria-live="polite">
        {done} / {total} asked{skipped ? ` · ${skipped} without a usable answer` : ''}
      </p>
    {/if}
    {#if error}
      <p class="text-sm text-destructive" role="alert">{error}</p>
    {/if}

    {#if items.length}
      <div class="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant={filter === 'all' ? 'default' : 'outline'}
          onclick={() => (filter = 'all')}>All {counts.all}</Button
        >
        {#each ORDER.filter((a) => counts[a]) as a (a)}
          <Button
            size="sm"
            variant={filter === a ? 'default' : 'outline'}
            onclick={() => (filter = a)}>{a} {counts[a]}</Button
          >
        {/each}
        <Button size="sm" variant="ghost" class="ml-auto" onclick={selectShown}>Select shown</Button
        >
        <Button
          size="sm"
          variant="ghost"
          disabled={!selected.size}
          onclick={() => (selected = new Set())}>Clear</Button
        >
      </div>
      <ul class="max-h-[60vh] divide-y overflow-y-auto rounded-md border">
        {#each shown as s (s.id)}
          <li class="flex items-start gap-3 p-3">
            <Checkbox
              class="mt-1"
              checked={selected.has(s.id)}
              onCheckedChange={() => toggle(s.id)}
              aria-label={`Apply ${s.action} to ${s.name}`}
            />
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-medium">{s.name}</span>
                <Badge variant={variant(s.action)}>{s.action}</Badge>
                {#if s.downgraded}
                  <Badge variant="outline">delete held back</Badge>
                {/if}
              </div>
              {#if s.reason}
                <p class="text-xs text-muted-foreground">{s.reason}</p>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
      <div class="flex items-center gap-2">
        <span class="text-sm text-muted-foreground tabular-nums">{selected.size} selected</span>
        <Button
          size="sm"
          class={`ml-auto ${items.some((s) => selected.has(s.id) && s.action === 'delete') ? DESTRUCTIVE_CLASS : ''}`}
          disabled={!selected.size || applying || running}
          onclick={apply}
        >
          Apply {selected.size}
        </Button>
      </div>
    {:else if !running && !done}
      <p class="py-6 text-center text-sm text-muted-foreground">
        Nothing asked yet. Run it after a scan so the facts are fresh.
      </p>
    {/if}
  </CardContent>
</Card>
