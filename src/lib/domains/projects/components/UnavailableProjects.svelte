<!--
	Projects whose folder is gone, reviewed one card at a time like a contact-merge
	flow so the list never crowds the page. Each card offers the same- or
	similar-named folders the backend found under the audit's scan folders;
	relinking repoints the row, removing drops it. Nothing on disk is touched.
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import ChevronLeft from '~icons/fa6-solid/chevron-left';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import FolderSearch from '~icons/fa6-solid/folder-open';
  import Link2 from '~icons/fa6-solid/link';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Trash2 from '~icons/fa6-solid/trash';
  import Unlink from '~icons/fa6-solid/link-slash';
  import { invokeClient } from '$lib/utils/invokeClient';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { projectService } from '$lib/domains/projects/services/projectService';
  import {
    createProjectsQuery,
    createUnavailableProjectsQuery,
  } from '$lib/domains/projects/queries/projectQueries';
  import type { UnavailableProject } from '$lib/domains/projects/generated/UnavailableProject';

  interface Props {
    /** False while the tab is hidden, so arrow keys there don't move cards unseen. */
    active?: boolean;
  }
  const { active = true }: Props = $props();

  const unavailableQuery = createUnavailableProjectsQuery();
  const projectsQuery = createProjectsQuery();

  const rows = $derived(unavailableQuery.data ?? []);
  /** Rows with a write in flight, so a double click cannot fire twice. */
  let busy = $state<Set<number>>(new Set());

  // One card at a time. `at` clamps, so resolving the last card lands on the new last.
  let index = $state(0);
  const at = $derived(Math.min(index, Math.max(rows.length - 1, 0)));
  const row = $derived(rows[at]);
  const go = (d: number) => (index = Math.min(Math.max(at + d, 0), rows.length - 1));

  // Swipe left = next, right = previous. Navigation only: relink and remove stay
  // explicit buttons so a stray drag can never change the registry.
  const SWIPE_PX = 80;
  let dragX = $state(0);
  let startX = $state<number | null>(null);

  function dragStart(e: PointerEvent) {
    if ((e.target as HTMLElement).closest('button,a,input')) return;
    startX = e.clientX;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function dragMove(e: PointerEvent) {
    if (startX !== null) dragX = e.clientX - startX;
  }
  function dragEnd() {
    if (startX === null) return;
    if (dragX <= -SWIPE_PX) go(1);
    else if (dragX >= SWIPE_PX) go(-1);
    startX = null;
    dragX = 0;
  }

  function onKey(e: KeyboardEvent) {
    const t = e.target as Element | null;
    // The tab strip owns ← → while it has focus.
    if (!active || t?.closest?.('input,textarea,select,[contenteditable],[role=tablist]')) return;
    if (e.metaKey || e.ctrlKey) return;
    if (e.key === 'ArrowRight') go(1);
    else if (e.key === 'ArrowLeft') go(-1);
  }

  const norm = (p: string) => p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();

  /** A POSIX path in a Windows registry: the row was imported from another machine. */
  const foreignPath = $derived(
    !!row && row.path.startsWith('/') && /Windows/i.test(globalThis.navigator?.userAgent ?? '')
  );

  /** A suggestion that is already registered would become a duplicate if relinked to. */
  const trackedBy = $derived.by(() => {
    const m = new Map<string, string>();
    for (const p of projectsQuery.data ?? []) m.set(norm(p.path), p.name);
    return m;
  });

  /** The project this row duplicates, when a suggestion is already registered. */
  const duplicateOf = $derived(
    row ? row.suggestions.map((p) => trackedBy.get(norm(p))).find(Boolean) : undefined
  );

  async function run(id: number, fn: () => Promise<void>) {
    busy = new Set(busy).add(id);
    try {
      await fn();
    } finally {
      const next = new Set(busy);
      next.delete(id);
      busy = next;
    }
  }

  function relink(r: UnavailableProject, path: string) {
    return run(r.id, async () => {
      try {
        await projectService.updateProject(String(r.id), { path });
        // Git state and size still describe the old folder until refreshed.
        await projectService.refreshProjectMetadata(String(r.id)).catch(() => {});
        toast.success(`${r.name} relinked`, path);
      } catch (e) {
        toast.error(`Could not relink ${r.name}`, e);
      }
    });
  }

  async function pickFolder(r: UnavailableProject) {
    try {
      const picked = await invokeClient.post<string | null>('select_directory');
      if (picked) await relink(r, picked);
    } catch (e) {
      toast.error('Could not open the folder picker', e);
    }
  }

  async function remove(r: UnavailableProject) {
    const ok = await confirmAction(
      'Removes the project from Portal. Nothing on disk is touched.',
      `Remove ${r.name}?`
    );
    if (!ok) return;
    await run(r.id, async () => {
      try {
        await projectService.deleteProject(String(r.id));
        toast.success(`${r.name} removed`);
      } catch (e) {
        toast.error(`Could not remove ${r.name}`, e);
      }
    });
  }
</script>

<svelte:window onkeydown={onKey} />

{#if row}
  <Card>
    <CardHeader class="pb-3">
      <div class="flex items-center gap-2">
        <CardTitle class="flex items-center gap-2 text-base">
          <Unlink class="h-4 w-4" /> Unavailable projects
          <Badge variant="destructive">{rows.length}</Badge>
        </CardTitle>
        <Button
          size="sm"
          variant="ghost"
          class="ml-auto"
          disabled={unavailableQuery.isFetching}
          onclick={() => unavailableQuery.refetch()}
        >
          <RefreshCw class={`mr-1 h-3 w-3 ${unavailableQuery.isFetching ? 'animate-spin' : ''}`} />
          Recheck
        </Button>
      </div>
      <p class="text-sm text-muted-foreground">
        Folder gone. Relink to where it went, or remove it. Swipe or press ← → to move between them.
      </p>
    </CardHeader>
    <CardContent class="space-y-3">
      <div
        role="group"
        aria-label={`Unavailable project ${at + 1} of ${rows.length}`}
        class="touch-pan-y space-y-3 rounded-md border p-4 select-none"
        class:motion-safe:transition-transform={startX === null}
        style:transform={`translateX(${dragX}px)`}
        style:opacity={1 - Math.min(Math.abs(dragX) / 400, 0.4)}
        onpointerdown={dragStart}
        onpointermove={dragMove}
        onpointerup={dragEnd}
        onpointercancel={dragEnd}
      >
        <div class="space-y-1">
          <p class="text-lg font-medium">{row.name}</p>
          <p class="font-mono text-xs break-all text-muted-foreground line-through">{row.path}</p>
          {#if foreignPath}
            <p class="text-xs text-muted-foreground">
              Linux-style path on Windows: this entry was imported from another machine.
            </p>
          {/if}
          {#if duplicateOf}
            <p class="text-xs text-foreground" role="note">
              Already tracked as <span class="font-medium">{duplicateOf}</span>. This entry is
              likely a stale duplicate; removing it is usually right.
            </p>
          {/if}
        </div>

        <div class="space-y-2">
          <p class="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Likely new home
          </p>
          {#each row.suggestions as path (path)}
            {@const owner = trackedBy.get(norm(path))}
            <div class="flex items-center gap-2">
              <span class="min-w-0 flex-1 truncate font-mono text-xs" title={path}>{path}</span>
              {#if owner}
                <Badge variant="outline">already tracked as {owner}</Badge>
              {:else}
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={busy.has(row.id)}
                  onclick={() => relink(row, path)}
                >
                  <Link2 class="mr-1 h-3 w-3" /> Relink
                </Button>
              {/if}
            </div>
          {:else}
            <p class="text-xs text-muted-foreground">
              No similar folder under the audit's scan folders — pick one or remove it.
            </p>
          {/each}
        </div>

        <div class="flex flex-wrap gap-2 border-t pt-3">
          <Button
            size="sm"
            variant="outline"
            disabled={busy.has(row.id)}
            onclick={() => pickFolder(row)}
          >
            <FolderSearch class="mr-1 h-3 w-3" /> Pick folder…
          </Button>
          <Button
            size="sm"
            variant="outline"
            class="ml-auto border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
            disabled={busy.has(row.id)}
            onclick={() => remove(row)}
          >
            <Trash2 class="mr-1 h-3 w-3" /> Remove
          </Button>
        </div>
      </div>

      <div class="flex items-center justify-between">
        <Button size="sm" variant="outline" disabled={at === 0} onclick={() => go(-1)}>
          <ChevronLeft class="mr-1 h-3 w-3" /> Previous
        </Button>
        <span class="text-sm text-muted-foreground tabular-nums">{at + 1} / {rows.length}</span>
        <Button size="sm" variant="outline" disabled={at >= rows.length - 1} onclick={() => go(1)}>
          Next <ChevronRight class="ml-1 h-3 w-3" />
        </Button>
      </div>
    </CardContent>
  </Card>
{/if}
