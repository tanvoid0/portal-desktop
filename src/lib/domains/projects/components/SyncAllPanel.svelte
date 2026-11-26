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
  import { openCoderSession } from '$lib/domains/coder/utils/launch';
  import { toast } from '$lib/utils/toast';
  import { syncAll, type SyncRow } from '../state/syncAll.svelte';
  import { commonDir } from '../utils/commonDir';
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
          path: p.path,
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
  // Env steps are vault work, which the coder can't do, so they don't count.
  const problems = (rs: SyncStepResult[]) =>
    rs.filter((s) => s.step !== 'env' && (s.status === 'attention' || s.status === 'failed'));

  const RULES = [
    'Inspect each repository (git status, git log) first, then resolve these.',
    'Do not push, force, reset --hard, rewrite published history or delete files without asking me.',
    'Never commit .env files, keys or other secrets.',
    'Show me what you are going to commit or merge before you do it.',
  ];
  const detail = (r: SyncRow) =>
    problems(results(r)).map((s) => `- ${s.step}: ${s.detail ?? LABEL[s.status]}`);

  // Only the final report carries paths, so this is empty while a run is in flight.
  const stuck = $derived(rows.filter((r) => r.path && problems(results(r)).length));

  // The coder pre-fills this and waits for the user to send it. The sheet stays
  // open in `syncAll`, so it is there again on returning to /projects.
  function resolve(row: SyncRow, path: string) {
    openCoderSession({
      workspaceRoot: path,
      projectId: String(row.id),
      prompt: [
        `"Sync all projects" could not finish syncing "${row.name}":`,
        ...detail(row),
        '',
        ...RULES,
      ].join('\n'),
    });
  }

  // One session rooted above every stuck repo, working through them in turn.
  function resolveAll() {
    const root = commonDir(stuck.map((r) => r.path!));
    if (!root) {
      toast.error('These projects share no parent folder. Resolve them one by one.');
      return;
    }
    openCoderSession({
      workspaceRoot: root,
      prompt: [
        `"Sync all projects" could not finish syncing ${stuck.length} projects. Work through them one at a time:`,
        ...stuck.flatMap((r) => ['', `${r.name} (${r.path})`, ...detail(r)]),
        '',
        ...RULES,
      ].join('\n'),
    });
  }

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
      <div class="flex flex-wrap items-center justify-between gap-2">
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
        {#if stuck.length > 1}
          <Button size="sm" onclick={resolveAll}>Resolve all with AI ({stuck.length})</Button>
        {/if}
      </div>
    </div>

    <ul class="flex-1 space-y-2 overflow-y-auto px-4">
      {#each shown as row (row.id)}
        {@const rs = results(row)}
        <li class="space-y-1 rounded-md border p-2 text-sm">
          <div class="flex items-center justify-between gap-2">
            <a href="/projects/{row.id}" class="font-medium hover:underline">{row.name}</a>
            {#if row.path && problems(rs).length}
              {@const path = row.path}
              <Button variant="outline" size="sm" onclick={() => resolve(row, path)}>
                Resolve with AI
              </Button>
            {/if}
          </div>
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
