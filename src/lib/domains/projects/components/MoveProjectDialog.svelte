<!--
	Relocate a project's folder (board #208, step 8 of #197's move engine). Preflight is an
	explicit step, not continuous-on-keystroke — it walks the source tree and probes the
	destination's in-use ports, too heavy to run on every character typed.
-->
<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Progress } from '$lib/components/ui/progress';
  import FolderPicker from '$lib/components/ui/folder-picker.svelte';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';
  import CircleXmark from '~icons/fa6-solid/circle-xmark';
  import Loader2 from '~icons/fa6-solid/spinner';
  import X from '~icons/fa6-solid/xmark';
  import FolderTree from '~icons/fa6-solid/folder-tree';
  import type { Project } from '$lib/domains/projects/types';
  import {
    preflightProjectMove,
    endPortProcess,
    MOVE_PROGRESS_EVENT,
    type MoveProgress,
    type PreflightResult,
    type PortWarning,
    type MoveSummary,
  } from '../api/projectMoveApi';
  import {
    startProjectMoveMutation,
    cancelProjectMoveMutation,
    removeMovedSourceMutation,
  } from '../queries/projectMoveQueries';
  import { fmtBytes } from '$lib/domains/disk/utils';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';

  interface Props {
    open?: boolean;
    project: Project;
  }
  let { open = $bindable(false), project }: Props = $props();

  let dest = $state('');
  let includeRegenerable = $state(false);
  let lastChecked = $state<{ dest: string; includeRegenerable: boolean } | null>(null);

  let preflight = $state<PreflightResult | null>(null);
  let preflightLoading = $state(false);
  let preflightError = $state<string | null>(null);
  let acknowledged = $state<Set<number>>(new Set());

  let endingIndex = $state<number | null>(null);
  let endErrors = $state<Record<number, string>>({});

  let starting = $state(false);
  let startError = $state<string | null>(null);
  let liveMoveId = $state<string | null>(null);
  let progress = $state<MoveProgress | null>(null);
  let summary = $state<MoveSummary | null>(null);
  let removing = $state(false);

  const checkStale = $derived(
    !lastChecked ||
      lastChecked.dest !== dest ||
      lastChecked.includeRegenerable !== includeRegenerable
  );
  const canStart = $derived(
    !!preflight &&
      !checkStale &&
      preflight.blockers.length === 0 &&
      preflight.warnings.every((_, i) => acknowledged.has(i))
  );

  function reset() {
    dest = '';
    includeRegenerable = false;
    lastChecked = null;
    preflight = null;
    preflightError = null;
    acknowledged = new Set();
    endErrors = {};
    starting = false;
    startError = null;
    liveMoveId = null;
    progress = null;
    summary = null;
  }

  $effect(() => {
    if (!open) reset();
  });

  // First progress event after our own `startProjectMoveMutation` call wins the id — the
  // dialog only ever starts one move for this project, so there's nothing to disambiguate
  // against in the common case. Two dialogs mid-move for two different projects at the
  // same time could in theory cross-talk here; `list_project_moves` stays the source of
  // truth, this is only the live progress readout.
  let unlisten: UnlistenFn | null = null;
  onMount(() => {
    listen<MoveProgress>(MOVE_PROGRESS_EVENT, (e) => {
      if (!starting) return;
      if (liveMoveId && e.payload.id !== liveMoveId) return;
      liveMoveId ??= e.payload.id;
      progress = e.payload;
    }).then((fn) => (unlisten = fn));
  });
  onDestroy(() => unlisten?.());

  async function runCheck() {
    if (!dest.trim() || preflightLoading) return;
    preflightLoading = true;
    preflightError = null;
    try {
      preflight = await preflightProjectMove(Number(project.id), dest.trim(), includeRegenerable);
      acknowledged = new Set();
      lastChecked = { dest, includeRegenerable };
    } catch (e) {
      preflight = null;
      preflightError = e instanceof Error ? e.message : String(e);
    } finally {
      preflightLoading = false;
    }
  }

  async function endProcess(index: number, w: PortWarning) {
    const ok = await confirmAction(
      `End ${w.process_name} (pid ${w.pid}) using port ${w.port}? Unsaved work in that process may be lost.`,
      'End process',
      { destructive: true, confirmLabel: 'End process' }
    );
    if (!ok) return;
    endingIndex = index;
    const rest = { ...endErrors };
    delete rest[index];
    endErrors = rest;
    try {
      await endPortProcess(w.port, w.pid, w.start_time);
      toast.success(`Ended ${w.process_name}`);
      await runCheck();
    } catch (e) {
      endErrors = { ...endErrors, [index]: e instanceof Error ? e.message : String(e) };
    } finally {
      endingIndex = null;
    }
  }

  async function handleStart() {
    if (!preflight || !canStart || starting) return;
    starting = true;
    startError = null;
    liveMoveId = null;
    progress = null;
    try {
      summary = await startProjectMoveMutation(
        Number(project.id),
        dest.trim(),
        includeRegenerable,
        preflight.warnings.length > 0
      );
      toast.success('Project moved');
    } catch (e) {
      startError = e instanceof Error ? e.message : String(e);
      toast.error(startError);
    } finally {
      starting = false;
    }
  }

  async function handleCancel() {
    if (!liveMoveId) return;
    await cancelProjectMoveMutation(liveMoveId);
    toast.info('Cancel requested — finishes the current file first, then rolls back');
  }

  async function handleRemoveSource() {
    if (!summary || removing) return;
    const ok = await confirmAction(
      `Permanently delete "${summary.source_path}" (sent to the Recycle Bin)? This can't be undone from here.`,
      'Remove old folder',
      { destructive: true, confirmLabel: 'Remove' }
    );
    if (!ok) return;
    removing = true;
    try {
      await removeMovedSourceMutation(summary.id);
      toast.success('Old folder removed');
      open = false;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      removing = false;
    }
  }

  const pct = $derived(
    progress && progress.total_bytes > 0
      ? (progress.bytes_done / progress.total_bytes) * 100
      : undefined
  );
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="flex max-h-[85vh] max-w-xl flex-col gap-0 overflow-hidden p-0">
    <Dialog.Header class="divider-edge-b divider-edge-full px-6 py-4">
      <Dialog.Title class="flex items-center gap-2 text-lg">
        <FolderTree class="h-5 w-5" />
        Move "{project.name}"
      </Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground">
        Copies the folder to a new location, then switches the project over once the copy is
        verified. Apps holding files open in the source (an editor, a running dev server) can block
        the lock — close them first if the check below flags it.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 py-4">
      {#if summary}
        <!-- Done -->
        <div class="rounded-md border border-border bg-muted/30 px-3 py-2 text-sm">
          Moved to <span class="font-mono text-xs">{summary.dest_path}</span>.
        </div>
        {#if summary.unrewritten.length > 0}
          <div
            class="rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-200"
          >
            <p class="mb-1 font-medium">These stored paths couldn't be rewritten automatically</p>
            <ul class="list-disc space-y-0.5 pl-4 font-mono text-xs">
              {#each summary.unrewritten as path (path)}
                <li>{path}</li>
              {/each}
            </ul>
            <p class="mt-1">Fix them by hand where they're used.</p>
          </div>
        {/if}
        <Button
          variant="outline"
          class="w-fit gap-2"
          disabled={removing}
          onclick={handleRemoveSource}
        >
          {#if removing}<Loader2 class="h-4 w-4 animate-spin" />{/if}
          Remove old folder…
        </Button>
      {:else if starting}
        <!-- In flight -->
        <div class="space-y-2">
          <p class="text-sm font-medium">
            {progress?.phase === 'verifying' ? 'Verifying…' : 'Copying…'}
          </p>
          <Progress value={pct} class="w-full" />
          {#if progress}
            <p class="text-xs text-muted-foreground tabular-nums">
              {progress.files_done} / {progress.file_count} files · {fmtBytes(progress.bytes_done)} /
              {fmtBytes(progress.total_bytes)}
            </p>
          {:else}
            <p class="text-xs text-muted-foreground">Starting…</p>
          {/if}
        </div>
        {#if startError}
          <div
            class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
          >
            {startError}
          </div>
        {/if}
      {:else}
        <!-- Setup -->
        <div class="space-y-1.5">
          <FolderPicker
            bind:value={dest}
            label="Destination"
            description="Where to move the project folder to."
            placeholder="/path/to/new/location"
            disabled={preflightLoading}
          />
        </div>

        <div
          class="flex items-center justify-between gap-4 rounded-md border border-border px-3 py-2"
        >
          <div>
            <Label for="include-regenerable">Include build / regenerable folders</Label>
            <p class="text-xs text-muted-foreground">
              node_modules, build output, etc. Off by default — they regenerate on the next
              install/build.
            </p>
          </div>
          <Switch
            id="include-regenerable"
            checked={includeRegenerable}
            disabled={preflightLoading}
            onCheckedChange={(v) => (includeRegenerable = v)}
          />
        </div>

        <Button
          variant="outline"
          class="w-fit gap-2"
          disabled={!dest.trim() || preflightLoading}
          onclick={runCheck}
        >
          {#if preflightLoading}<Loader2 class="h-4 w-4 animate-spin" />{/if}
          Check
        </Button>

        {#if preflightError}
          <div
            class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
          >
            {preflightError}
          </div>
        {/if}

        {#if preflight}
          <p class="text-xs text-muted-foreground tabular-nums">
            {preflight.file_count} files · {fmtBytes(preflight.total_bytes)}
            {#if preflight.available_bytes > 0}
              · {fmtBytes(preflight.available_bytes)} free at destination
            {/if}
          </p>

          {#if preflight.nested_projects.length > 0}
            <p class="text-xs text-muted-foreground">
              Also registered under this path, and moving with it: {preflight.nested_projects
                .map((p) => p.name)
                .join(', ')}
            </p>
          {/if}

          {#if preflight.blockers.length > 0}
            <div class="space-y-1.5">
              {#each preflight.blockers as b (b.kind)}
                <div
                  class="flex items-start gap-2 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
                >
                  <CircleXmark class="mt-0.5 h-4 w-4 shrink-0" />
                  {b.detail}
                </div>
              {/each}
            </div>
          {/if}

          {#if preflight.warnings.length > 0}
            <div class="space-y-1.5">
              {#each preflight.warnings as w, i (w.port)}
                <div
                  class="flex items-start gap-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm dark:border-amber-800 dark:bg-amber-950"
                >
                  <TriangleExclamation
                    class="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400"
                  />
                  <div class="min-w-0 flex-1 space-y-1">
                    <p class="text-amber-800 dark:text-amber-200">
                      Port {w.port}: {w.process_name} (pid {w.pid}) — {w.reason}
                    </p>
                    {#if endErrors[i]}
                      <p class="text-xs text-destructive">{endErrors[i]}</p>
                    {/if}
                    <div class="flex items-center gap-2">
                      <label
                        class="flex items-center gap-1.5 text-xs text-amber-800 dark:text-amber-200"
                      >
                        <Checkbox
                          checked={acknowledged.has(i)}
                          onCheckedChange={(v) => {
                            const next = new Set(acknowledged);
                            if (v === true) next.add(i);
                            else next.delete(i);
                            acknowledged = next;
                          }}
                        />
                        Proceed anyway
                      </label>
                      <Button
                        size="sm"
                        variant="outline"
                        class="h-7 gap-1 text-xs"
                        disabled={!w.killable || endingIndex === i}
                        title={!w.killable ? w.reason : undefined}
                        onclick={() => endProcess(i, w)}
                      >
                        {#if endingIndex === i}
                          <Loader2 class="h-3 w-3 animate-spin" />
                        {:else}
                          <X class="h-3 w-3" />
                        {/if}
                        End process
                      </Button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          {#if !includeRegenerable && preflight.skipped_dirs.length > 0}
            <div
              class="space-y-1 rounded-md border border-border px-3 py-2 text-xs text-muted-foreground"
            >
              <p class="font-medium">Excluded (regenerable):</p>
              {#each preflight.skipped_dirs as dir (dir.rel)}
                <p class="flex justify-between gap-2 tabular-nums">
                  <span class="truncate font-mono">{dir.rel}</span>
                  <span class="shrink-0">{fmtBytes(dir.bytes)}</span>
                </p>
              {/each}
            </div>
          {/if}
        {/if}
      {/if}
    </div>

    <Dialog.Footer class="divider-edge-t divider-edge-full px-6 py-4">
      {#if starting}
        <Button variant="outline" disabled={!liveMoveId} onclick={handleCancel}>Cancel move</Button>
      {:else if summary}
        <Button onclick={() => (open = false)}>Done</Button>
      {:else}
        <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button disabled={!canStart} onclick={handleStart}>Start move</Button>
      {/if}
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
