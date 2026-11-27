<!-- External drive sync: pick a drive, see which side of each project is behind, copy or merge. -->
<script module lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import { queryClient } from '$lib/domains/shared/query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { formatFileSize } from '$lib/utils/fileSize';
  import { applyDriveSync } from '../api/driveSyncApi';
  import type { DriveSyncDirection } from '../generated/DriveSyncDirection';

  interface Outcome {
    lines: string[];
    problem: boolean;
  }

  // Module level so a long bulk copy survives leaving the page and coming back.
  const outcomes = $state<Record<string, Outcome>>({});
  const selected = new SvelteSet<number>();
  const job = $state({
    busy: null as number | null,
    bulk: false,
    stop: false,
    done: 0,
    total: 0,
    summary: '',
  });
  const outcomeKey = (drive: string, id: number) => `${drive}|${id}`;

  // A status walks both copies; all projects at once would thrash the drive.
  let active = 0;
  const waiting: (() => void)[] = [];
  async function limited<T>(signal: AbortSignal, task: () => Promise<T>): Promise<T> {
    if (active >= 4) await new Promise<void>((go) => waiting.push(go));
    else active++;
    try {
      signal.throwIfAborted();
      return await task();
    } finally {
      // Hand the slot straight to the next waiter.
      const next = waiting.shift();
      if (next) next();
      else active--;
    }
  }

  async function apply(
    drive: string,
    id: number,
    direction: DriveSyncDirection,
    force = false
  ): Promise<{ ok: boolean; error?: string }> {
    job.busy = id;
    const status = queryKeys.driveSync.status(drive, id);
    try {
      const result = await applyDriveSync(drive, id, direction, force);
      queryClient.setQueryData(status, result.status);
      const problems = [...result.failures, ...result.conflicts];
      outcomes[outcomeKey(drive, id)] = {
        lines: [
          `Copied ${result.filesCopied} files (${formatFileSize(result.bytesCopied, 1)})`,
          ...problems,
          ...result.notes,
        ],
        problem: problems.length > 0,
      };
      return { ok: problems.length === 0 };
    } catch (e) {
      const error = getErrorMessage(e, 'Sync failed');
      outcomes[outcomeKey(drive, id)] = { lines: [error], problem: true };
      void queryClient.invalidateQueries({ queryKey: status });
      return { ok: false, error };
    } finally {
      job.busy = null;
      void queryClient.invalidateQueries({ queryKey: queryKeys.driveSync.drives });
    }
  }

  async function copyAll(drive: string, ids: number[], skipped: string) {
    Object.assign(job, { bulk: true, stop: false, done: 0, total: ids.length, summary: '' });
    let problems = 0;
    for (const id of ids) {
      if (job.stop) break;
      if (!(await apply(drive, id, 'toDrive')).ok) problems++;
      job.done++;
    }
    job.bulk = false;
    job.summary = [
      `${job.stop ? 'Stopped after' : 'Finished'} ${job.done} of ${job.total}`,
      problems ? `${problems} with problems (see the rows)` : '',
      skipped,
    ]
      .filter(Boolean)
      .join('. ');
    if (problems) toast.warning(job.summary);
    else toast.success(job.summary);
  }
</script>

<script lang="ts">
  import { createQueries, createQuery } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import { PageEmpty, PageError, PageLoading } from '$lib/components/shell';
  import { confirmAction } from '$lib/utils/confirm';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import { fetchDriveStatus, fetchSyncDrives } from '../api/driveSyncApi';
  import { createProjectsQuery } from '../queries/projectQueries';
  import type { DriveProjectStatus } from '../generated/DriveProjectStatus';

  const STORAGE_KEY = 'drive-sync:drive';

  function remembered(): string {
    try {
      return localStorage.getItem(STORAGE_KEY) ?? '';
    } catch {
      return '';
    }
  }

  let chosen = $state(remembered());
  let search = $state('');

  function choose(value: string) {
    chosen = value;
    job.summary = '';
    try {
      localStorage.setItem(STORAGE_KEY, value);
    } catch {
      // Remembering the drive is a convenience only.
    }
  }

  const drivesQuery = createQuery(() => ({
    queryKey: queryKeys.driveSync.drives,
    queryFn: fetchSyncDrives,
    staleTime: 0,
  }));
  const drives = $derived(drivesQuery.data ?? []);
  // The remembered drive if it is plugged in, else the one already holding a mirror.
  const drive = $derived(
    drives.some((d) => d.mountPoint === chosen)
      ? chosen
      : (drives.find((d) => d.hasMirror)?.mountPoint ?? '')
  );
  const driveOptions = $derived(
    drives.map((d) => ({
      value: d.mountPoint,
      label: [
        `${d.mountPoint} ${d.label}`.trim(),
        `${formatFileSize(d.availableBytes, 1)} free`,
        d.removable ? 'external' : '',
      ]
        .filter(Boolean)
        .join(' · '),
    }))
  );

  const projectsQuery = createProjectsQuery();
  const projects = $derived(
    (projectsQuery.data ?? [])
      .filter((p) => !p.archived)
      .sort((a, b) => a.name.localeCompare(b.name))
  );

  const statuses = createQueries(() => {
    const mount = drive;
    return {
      queries: projects.map((p) => {
        const id = Number(p.id);
        return {
          queryKey: queryKeys.driveSync.status(mount, id),
          queryFn: ({ signal }: { signal: AbortSignal }) =>
            limited(signal, () => fetchDriveStatus(mount, id)),
          enabled: !!mount,
          retry: false,
          refetchOnWindowFocus: false,
        };
      }),
    };
  });

  const rows = $derived(
    projects.map((p, i) => ({
      id: Number(p.id),
      name: p.name,
      path: p.path,
      status: statuses[i]?.data,
      error: statuses[i]?.isError ? getErrorMessage(statuses[i].error, 'Status failed') : null,
    }))
  );
  const shown = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return q
      ? rows.filter((r) => r.name.toLowerCase().includes(q) || r.path.toLowerCase().includes(q))
      : rows;
  });
  const checking = $derived(drive ? rows.filter((r) => !r.status && !r.error).length : 0);
  const picked = $derived(rows.filter((r) => selected.has(r.id)));
  const copyable = $derived(
    picked.filter((r) => r.status?.state === 'notOnDrive' || r.status?.state === 'driveBehind')
  );
  const needAttention = $derived(
    picked.filter(
      (r) =>
        r.error ||
        ['localBehind', 'diverged', 'unsupported', 'missing', 'error'].includes(
          r.status?.state ?? ''
        )
    ).length
  );
  const stillChecking = $derived(picked.filter((r) => !r.status && !r.error).length);
  const allShown = $derived(shown.length > 0 && shown.every((r) => selected.has(r.id)));
  const someShown = $derived(!allShown && shown.some((r) => selected.has(r.id)));
  const idle = $derived(job.busy === null && !job.bulk);

  function toggle(id: number, on: boolean) {
    if (on) selected.add(id);
    else selected.delete(id);
  }

  /** "3 commits and 2 files " (trailing space), or nothing. */
  function amount(commits: number, files: number): string {
    const parts = [
      commits ? `${commits} commit${commits === 1 ? '' : 's'}` : '',
      files ? `${files} file${files === 1 ? '' : 's'}` : '',
    ].filter(Boolean);
    return parts.length ? `${parts.join(' and ')} ` : '';
  }

  function describe(s: DriveProjectStatus): string {
    const pc = amount(s.localAhead, s.localOnlyFiles);
    const dr = amount(s.driveAhead, s.driveOnlyFiles);
    switch (s.state) {
      case 'notOnDrive':
        return 'Not on drive';
      case 'inSync':
        return 'In sync';
      case 'driveBehind':
        return `Drive is ${pc}behind`;
      case 'localBehind':
        return `PC is ${dr}behind`;
      case 'diverged':
        return `Diverged: PC has ${pc || 'changes '}the drive lacks, drive has ${dr || 'changes '}the PC lacks`;
      case 'covered':
        return `Included in ${s.coveredBy}`;
      case 'unsupported':
        return 'Not supported';
      case 'missing':
        return 'Folder missing on this PC';
      case 'error':
        return 'Cannot check the drive copy';
    }
  }

  function details(s: DriveProjectStatus): string {
    const unpulled = s.state === 'localBehind' || s.state === 'diverged';
    return [
      describe(s),
      s.localDirty ? `${s.localDirty} uncommitted` : '',
      unpulled && s.driveDirty ? `${s.driveDirty} uncommitted on drive` : '',
      s.note ?? '',
    ]
      .filter(Boolean)
      .join(' · ');
  }

  const branchList = (s: DriveProjectStatus) =>
    s.branches.map((b) => `${b.name}: PC +${b.localAhead}, drive +${b.driveAhead}`).join('\n');

  async function sync(id: number, direction: DriveSyncDirection, force = false) {
    const { error } = await apply(drive, id, direction, force);
    if (error) toast.error(error);
  }

  async function overwrite(id: number, name: string) {
    const mount = drive;
    const ok = await confirmAction(
      `Replace the drive copy of "${name}" with the one on this PC? Commits and edits that exist only on the drive will be permanently deleted.`,
      'Overwrite drive',
      { destructive: true, confirmLabel: 'Overwrite drive' }
    );
    if (!ok) return;
    const { error } = await apply(mount, id, 'toDrive', true);
    if (error) toast.error(error);
  }

  function copySelected() {
    const skipped = [
      needAttention ? `${needAttention} skipped: need attention` : '',
      stillChecking ? `${stillChecking} skipped: still checking` : '',
    ]
      .filter(Boolean)
      .join(', ');
    void copyAll(
      drive,
      copyable.map((r) => r.id),
      skipped
    );
  }

  function refresh() {
    void queryClient.invalidateQueries({ queryKey: queryKeys.driveSync.all });
  }
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center gap-2">
    <div class="w-full sm:w-96">
      <Select
        options={driveOptions}
        value={drive}
        onValueChange={choose}
        aria-label="Drive to sync with"
        placeholder="Choose a drive…"
        disabled={!idle}
      />
    </div>
    <Button variant="outline" onclick={refresh} disabled={!idle}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
  </div>
  <p class="text-sm text-muted-foreground">
    <code>.env</code> and other ignored files are copied to the drive in plain text. Build folders are
    skipped.
  </p>

  {#if drivesQuery.isPending || projectsQuery.isPending}
    <PageLoading message="Looking for drives..." />
  {:else if drivesQuery.isError || projectsQuery.isError}
    <PageError
      title="Could not load drive sync"
      message={getErrorMessage(drivesQuery.error ?? projectsQuery.error, 'Request failed')}
      onRetry={refresh}
    />
  {:else if !drive}
    <PageEmpty
      icon={HardDrive}
      title={drives.length ? 'Choose a drive' : 'No drive found'}
      description={drives.length
        ? 'Pick the drive to mirror your projects to.'
        : 'Connect a drive other than the system drive, then refresh.'}
    />
  {:else}
    <div class="flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <Checkbox
          checked={allShown}
          indeterminate={someShown}
          onCheckedChange={(on) => shown.forEach((r) => toggle(r.id, on === true))}
          aria-label="Select all shown projects"
        />
        <span class="text-sm tabular-nums">{picked.length} selected</span>
      </div>
      <Input
        bind:value={search}
        placeholder="Filter projects..."
        aria-label="Filter projects"
        class="w-full sm:max-w-xs"
      />
      {#if job.bulk}
        <span class="text-sm tabular-nums" role="status">
          Copying {Math.min(job.done + 1, job.total)} of {job.total}
        </span>
        <Button variant="outline" onclick={() => (job.stop = true)} disabled={job.stop}>
          {job.stop ? 'Stopping after this one' : 'Stop'}
        </Button>
      {:else}
        <Button onclick={copySelected} disabled={!idle || copyable.length === 0}>
          Copy / update selected ({copyable.length})
        </Button>
        {#if needAttention}
          <span class="text-sm text-muted-foreground">{needAttention} need attention</span>
        {/if}
      {/if}
      {#if checking}
        <span class="text-sm text-muted-foreground tabular-nums">Checking {checking}...</span>
      {/if}
    </div>
    {#if job.summary}
      <p class="text-sm" role="status">{job.summary}</p>
    {/if}

    <ul class="divide-y rounded-md border">
      {#each shown as row (row.id)}
        {@const s = row.status}
        {@const outcome = outcomes[outcomeKey(drive, row.id)]}
        <li class="flex flex-wrap items-start gap-3 p-3">
          <Checkbox
            class="mt-0.5"
            checked={selected.has(row.id)}
            onCheckedChange={(on) => toggle(row.id, on === true)}
            aria-label={`Select ${row.name}`}
          />
          <div class="min-w-0 flex-1 basis-56 space-y-0.5">
            <p class="truncate text-sm font-medium" title={row.name}>{row.name}</p>
            <p class="truncate text-xs text-muted-foreground" title={row.path}>{row.path}</p>
            {#if job.busy === row.id}
              <p class="text-xs" role="status">Syncing...</p>
            {:else if row.error}
              <p class="text-xs wrap-anywhere text-destructive">{row.error}</p>
            {:else if s}
              <p
                class="text-xs wrap-anywhere {s.state === 'error' ? 'text-destructive' : ''}"
                title={branchList(s)}
              >
                {details(s)}
              </p>
            {:else}
              <p class="text-xs text-muted-foreground">Checking...</p>
            {/if}
            {#if outcome}
              <ul
                class="max-h-32 overflow-y-auto text-xs wrap-anywhere {outcome.problem
                  ? 'text-destructive'
                  : 'text-muted-foreground'}"
              >
                {#each outcome.lines as line, i (i)}
                  <li>{line}</li>
                {/each}
              </ul>
            {/if}
          </div>
          <div class="flex flex-wrap gap-2">
            {#if s?.state === 'notOnDrive' || s?.state === 'driveBehind'}
              <Button size="sm" disabled={!idle} onclick={() => sync(row.id, 'toDrive')}>
                {s.state === 'notOnDrive' ? 'Copy to drive' : 'Update drive'}
              </Button>
            {:else if s?.state === 'localBehind' || s?.state === 'diverged' || s?.state === 'error'}
              {#if s.state !== 'error'}
                <Button size="sm" disabled={!idle} onclick={() => sync(row.id, 'toLocal')}>
                  {s.state === 'localBehind' ? 'Update this PC' : 'Merge into this PC'}
                </Button>
              {/if}
              <Button
                size="sm"
                variant="outline"
                disabled={!idle}
                onclick={() => overwrite(row.id, row.name)}
              >
                Overwrite drive
              </Button>
            {/if}
          </div>
        </li>
      {:else}
        <li class="p-3 text-sm text-muted-foreground">No projects match.</li>
      {/each}
    </ul>
  {/if}
</div>
