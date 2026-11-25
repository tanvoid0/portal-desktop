<!-- "Sync all" progress and result, in a sheet. Reads the module-level syncAll singleton. -->
<script lang="ts">
  import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
  } from '$lib/components/ui/sheet';
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Progress } from '$lib/components/ui/progress';
  import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
  import CircleCheck from '~icons/fa6-solid/circle-check';
  import CircleMinus from '~icons/fa6-solid/circle-minus';
  import CircleXmark from '~icons/fa6-solid/circle-xmark';
  import Spinner from '~icons/fa6-solid/spinner';
  import Hand from '~icons/fa6-solid/hand';
  import { syncAll, type SyncRow } from '../state/syncAll.svelte';
  import type { SyncStep } from '../generated/SyncStep';
  import type { SyncStepResult } from '../generated/SyncStepResult';
  import type { SyncStepStatus } from '../generated/SyncStepStatus';

  const STEPS: SyncStep[] = ['env', 'fetch', 'pull', 'push', 'mark'];
  const LABEL: Record<SyncStepStatus, string> = {
    running: 'Running',
    ok: 'OK',
    skipped: 'Skipped',
    attention: 'Needs your hand',
    failed: 'Failed',
  };
  const ICON = {
    running: Spinner,
    ok: CircleCheck,
    skipped: CircleMinus,
    attention: Hand,
    failed: CircleXmark,
  };
  const VARIANT: Record<SyncStepStatus, BadgeVariant> = {
    running: 'secondary',
    ok: 'outline',
    skipped: 'outline',
    attention: 'default',
    failed: 'destructive',
  };
  const PHASE = {
    env: 'Backing up env files',
    git: 'Syncing repositories',
    snapshot: 'Uploading snapshot',
    done: 'Done',
  };

  let filter = $state<'attention' | 'all'>('attention');

  // Prefer the final report once it lands; live rows until then.
  const rows = $derived<SyncRow[]>(
    syncAll.report
      ? syncAll.report.projects.map((p) => ({
          id: p.id,
          name: p.name,
          steps: Object.fromEntries(p.steps.map((s) => [s.step, s])),
        }))
      : Object.values(syncAll.rows)
  );
  const results = (r: SyncRow) =>
    STEPS.map((s) => r.steps[s]).filter((s): s is SyncStepResult => !!s);
  const shown = $derived(
    filter === 'all'
      ? rows
      : rows.filter((r) =>
          results(r).some(
            (s) => s.status === 'attention' || s.status === 'failed' || s.status === 'running'
          )
        )
  );
  const percent = $derived(
    syncAll.total > 0 ? Math.round((syncAll.done / syncAll.total) * 100) : 0
  );
  const finished = $derived(syncAll.phase === 'done');
</script>

{#snippet stepBadge(s: SyncStepResult)}
  {@const Icon = ICON[s.status]}
  <Badge variant={VARIANT[s.status]} class="gap-1" title={s.detail ?? ''}>
    <Icon class={s.status === 'running' ? 'animate-spin' : ''} />
    {s.step}: {LABEL[s.status]}
  </Badge>
{/snippet}

<Sheet bind:open={syncAll.open}>
  <SheetContent
    class="flex w-full max-w-[min(100vw-2rem,46rem)] flex-col sm:max-w-[min(100vw-2rem,46rem)]"
  >
    <SheetHeader>
      <SheetTitle>Sync all projects</SheetTitle>
      <SheetDescription>{PHASE[syncAll.phase]}</SheetDescription>
    </SheetHeader>

    <div class="space-y-2 px-4">
      <Progress value={percent} class="w-full" />
      <div class="flex items-center justify-between">
        <p class="text-xs text-muted-foreground tabular-nums">{syncAll.done} / {syncAll.total}</p>
        {#if finished || !syncAll.running}
          <Button variant="outline" size="sm" onclick={() => (syncAll.open = false)}>Close</Button>
        {:else}
          <Button variant="outline" size="sm" onclick={() => syncAll.cancel()}>Cancel</Button>
        {/if}
      </div>
      <ToggleGroup
        type="single"
        variant="outline"
        size="sm"
        bind:value={filter}
        aria-label="Filter projects"
      >
        <ToggleGroupItem value="attention">Needs attention</ToggleGroupItem>
        <ToggleGroupItem value="all">All ({rows.length})</ToggleGroupItem>
      </ToggleGroup>
    </div>

    <ul class="flex-1 space-y-2 overflow-y-auto px-4">
      {#each shown as row (row.id)}
        {@const rs = results(row)}
        <li class="space-y-1 rounded-md border p-2 text-sm">
          <a href="/projects/{row.id}" class="font-medium hover:underline">{row.name}</a>
          <div class="flex flex-wrap gap-1">
            {#each rs as s (s.step)}{@render stepBadge(s)}{/each}
          </div>
          {#each rs.filter((s) => s.status !== 'ok' && s.detail) as s (s.step)}
            <p class="text-xs text-muted-foreground">{s.step}: {s.detail}</p>
          {/each}
        </li>
      {:else}
        <li class="text-sm text-muted-foreground">
          {syncAll.running ? 'Nothing needs attention so far.' : 'Nothing needs attention.'}
        </li>
      {/each}
    </ul>

    {#if syncAll.report}
      <SheetFooter class="flex-col items-start gap-1 text-xs text-muted-foreground">
        {#if syncAll.report.cancelled}<p>Cancelled before every project was synced.</p>{/if}
        {#if syncAll.report.envNote}<p>Env: {syncAll.report.envNote}</p>{/if}
        <p>
          Snapshot: {LABEL[syncAll.report.snapshot.status]}{syncAll.report.snapshot.detail
            ? ` — ${syncAll.report.snapshot.detail}`
            : ''}
        </p>
      </SheetFooter>
    {/if}
  </SheetContent>
</Sheet>
