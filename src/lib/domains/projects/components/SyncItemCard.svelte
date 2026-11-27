<!-- One project in a sync: what happened, and the quickest way to clear what is left. -->
<script lang="ts">
  import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import CircleCheck from '~icons/fa6-solid/circle-check';
  import CircleMinus from '~icons/fa6-solid/circle-minus';
  import CircleXmark from '~icons/fa6-solid/circle-xmark';
  import Spinner from '~icons/fa6-solid/spinner';
  import Hand from '~icons/fa6-solid/hand';
  import Rotate from '~icons/fa6-solid/rotate';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { setProjectPushOk } from '../api/projectApi';
  import { syncAll, type SyncRow } from '../state/syncAll.svelte';
  import { syncTriage } from '../state/syncTriage.svelte';
  import { openSteps } from '../utils/syncTriage';
  import { coderProblems, openInCoder } from '../utils/syncCoder';
  import SyncTriagePanel from './SyncTriagePanel.svelte';
  import EnvBackupSection from './EnvBackupSection.svelte';
  import type { SyncStep } from '../generated/SyncStep';
  import type { SyncStepResult } from '../generated/SyncStepResult';
  import type { SyncStepStatus } from '../generated/SyncStepStatus';

  interface Props {
    row: SyncRow;
  }
  let { row }: Props = $props();

  const STEPS: SyncStep[] = ['env', 'fetch', 'pull', 'push', 'mark'];
  const NAME: Record<SyncStep, string> = {
    env: 'Env',
    fetch: 'Fetch',
    pull: 'Pull',
    push: 'Push',
    mark: 'Changes',
    snapshot: 'Snapshot',
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

  /** What is wrong, in words that say what to do about it. */
  function title(s: SyncStepResult): string {
    if (s.status === 'failed') return `${NAME[s.step]} failed`;
    if (s.step === 'mark') return 'Uncommitted changes';
    if (s.step === 'push') return 'Not pushed';
    if (s.step === 'pull') return 'Diverged from upstream';
    return 'Env files need a look';
  }

  const results = $derived(STEPS.map((s) => row.steps[s]).filter((s): s is SyncStepResult => !!s));
  // Set once the final report lands; nothing can be acted on before that.
  const project = $derived(syncAll.project(row.id));
  const open = $derived(project ? openSteps(results) : []);
  const busy = $derived(syncAll.running || syncAll.busy.includes(row.id));
  const triaged = $derived(!!syncTriage.items[row.id]);
  const dirty = $derived((project?.dirty ?? 0) > 0);
  const envOpen = $derived(syncAll.openEnv.includes(row.id));

  async function rebase() {
    const ok = await confirmAction(
      `Replay the unpushed commits of "${row.name}" on top of its upstream? If it conflicts, the rebase is undone.`,
      'Rebase onto upstream'
    );
    if (ok) await syncAll.quick(row.id, { rebase: true });
  }

  async function alwaysPush() {
    try {
      await setProjectPushOk(row.id, true);
    } catch (err) {
      toast.error(`Could not allow pushing: ${getErrorMessage(err)}`);
      return;
    }
    await syncAll.quick(row.id, { push: true });
  }
</script>

<li class="space-y-2 rounded-md border bg-card p-3 text-sm">
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
    <a
      href="/projects/{row.id}"
      class="max-w-full min-w-0 truncate font-medium hover:underline"
      title={row.path ?? row.name}>{row.name}</a
    >
    {#if !results.length}
      <span class="text-xs text-muted-foreground">Not synced yet</span>
    {/if}
    <div class="flex flex-wrap gap-1">
      {#each results as s (s.step)}
        {@const Icon = ICON[s.status]}
        <Badge variant={VARIANT[s.status]} class="gap-1" title={s.detail ?? s.status}>
          <Icon class={s.status === 'running' ? 'animate-spin' : ''} />
          {NAME[s.step]}
          <span class="sr-only">{s.status}</span>
        </Badge>
      {/each}
    </div>
    <div class="ml-auto flex shrink-0 gap-1">
      {#if project && coderProblems(project).length}
        <Button size="sm" variant="ghost" onclick={() => openInCoder(project)}>Open in Coder</Button
        >
      {/if}
      <Button
        size="sm"
        variant="ghost"
        aria-expanded={envOpen}
        onclick={() => syncAll.toggleEnv(row.id)}
      >
        Env files
      </Button>
      <Button
        size="sm"
        variant="outline"
        disabled={busy}
        aria-label="Sync {row.name}"
        onclick={() => syncAll.recheck(row.id)}
      >
        <Rotate class="mr-1 h-3 w-3 {syncAll.busy.includes(row.id) ? 'animate-spin' : ''}" />
        Sync
      </Button>
    </div>
  </div>

  {#each open as s (s.step)}
    <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
      <p class="min-w-0 text-xs wrap-anywhere">
        <span class="font-medium {s.status === 'failed' ? 'text-destructive' : ''}">{title(s)}</span
        >
        {#if s.detail}<span class="text-muted-foreground"> · {s.detail}</span>{/if}
      </p>
      {#if s.step === 'push' && s.status === 'attention' && (project?.behind ?? 0) <= 0}
        <Button
          size="sm"
          variant="outline"
          disabled={busy}
          onclick={() => syncAll.quick(row.id, { push: true })}>Push now</Button
        >
        <Button size="sm" variant="ghost" disabled={busy} onclick={alwaysPush}>
          Always push this project
        </Button>
      {:else if s.step === 'pull' && s.status === 'attention'}
        <Button
          size="sm"
          variant="outline"
          disabled={busy || dirty}
          title={dirty ? 'Commit or stash the changes first' : undefined}
          onclick={rebase}>Rebase onto upstream</Button
        >
      {:else if s.step === 'env'}
        {#if !envOpen}
          <Button size="sm" variant="outline" onclick={() => syncAll.toggleEnv(row.id)}>
            Review env files
          </Button>
        {/if}
      {:else if s.status === 'failed'}
        <Button size="sm" variant="outline" disabled={busy} onclick={() => syncAll.recheck(row.id)}>
          Retry
        </Button>
      {/if}
    </div>
  {/each}

  {#if triaged || open.some((s) => s.step === 'mark' && s.status === 'attention')}
    <SyncTriagePanel projectId={row.id} disabled={busy} />
  {/if}

  {#if envOpen}
    <EnvBackupSection
      projectId={row.id}
      projectName={row.name}
      title="Env files"
      description="Back these up into the vault, or write the vault copy back to disk."
      onChanged={() => syncAll.recheck(row.id)}
    />
  {/if}
</li>
