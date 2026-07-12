<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Badge } from "$lib/components/ui/badge";
  import { Check, X, FileEdit, Pencil, ChevronDown, ChevronUp } from "@lucide/svelte";
  import { coderService } from "../services/coderService.js";
  import type { FileChange, Hunk } from "../types.js";

  interface Props {
    changes: FileChange[];
    onRefresh: () => void | Promise<void>;
  }

  let { changes, onRefresh }: Props = $props();

  let editing = $state<string | null>(null);
  let draft = $state("");
  let focusChangeId = $state<string | null>(null);
  let focusHunkIndex = $state<number | null>(null);
  let reviewedHunks = $state(new Set<string>());

  const pendingChanges = $derived(changes.filter((c) => c.status === "pending"));

  const focusChange = $derived(
    focusChangeId ? changes.find((c) => c.id === focusChangeId) ?? null : null,
  );

  const focusPosition = $derived.by(() => {
    if (!focusChangeId || focusHunkIndex === null) return null;
    const changeIdx = pendingChanges.findIndex((c) => c.id === focusChangeId);
    if (changeIdx < 0) return null;
    const change = pendingChanges[changeIdx];
    const hunkIdx = change.hunks.findIndex((h) => h.index === focusHunkIndex);
    if (hunkIdx < 0) return null;
    return {
      changeIdx,
      hunkIdx,
      totalChanges: pendingChanges.length,
      totalHunks: change.hunks.length,
    };
  });

  function hunkKey(changeId: string, hunkIndex: number): string {
    return `${changeId}:${hunkIndex}`;
  }

  function isReviewed(changeId: string, hunkIndex: number): boolean {
    return reviewedHunks.has(hunkKey(changeId, hunkIndex));
  }

  function isFocused(changeId: string, hunkIndex: number): boolean {
    return focusChangeId === changeId && focusHunkIndex === hunkIndex;
  }

  function findNextUnreviewed(
    afterChangeId?: string,
    afterHunkIndex?: number,
  ): { changeId: string; hunkIndex: number } | null {
    const list = pendingChanges;
    if (list.length === 0) return null;

    let startChangeIdx = 0;
    if (afterChangeId !== undefined) {
      const idx = list.findIndex((c) => c.id === afterChangeId);
      if (idx >= 0) startChangeIdx = idx;
    }

    for (let ci = startChangeIdx; ci < list.length; ci++) {
      const change = list[ci];
      let startHunk = 0;
      if (ci === startChangeIdx && afterHunkIndex !== undefined) {
        const currentIdx = change.hunks.findIndex((h) => h.index === afterHunkIndex);
        startHunk = currentIdx >= 0 ? currentIdx + 1 : 0;
      }

      for (let hi = startHunk; hi < change.hunks.length; hi++) {
        const hunk = change.hunks[hi];
        if (!isReviewed(change.id, hunk.index)) {
          return { changeId: change.id, hunkIndex: hunk.index };
        }
      }
    }

    return null;
  }

  function syncFocus() {
    if (focusChangeId && focusHunkIndex !== null) {
      const change = changes.find((c) => c.id === focusChangeId);
      const hunkExists = change?.hunks.some((h) => h.index === focusHunkIndex);
      if (change?.status === "pending" && hunkExists) return;
    }

    const first = findNextUnreviewed();
    if (first) {
      focusChangeId = first.changeId;
      focusHunkIndex = first.hunkIndex;
    } else {
      focusChangeId = null;
      focusHunkIndex = null;
    }
  }

  async function scrollToFocus() {
    await tick();
    if (!focusChangeId || focusHunkIndex === null) return;
    const el = document.querySelector(
      `[data-hunk-id="${focusChangeId}:${focusHunkIndex}"]`,
    );
    el?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }

  async function refresh() {
    await onRefresh();
    syncFocus();
    await scrollToFocus();
  }

  async function finalizeChangeIfReviewed(change: FileChange) {
    if (change.hunks.some((h) => !isReviewed(change.id, h.index))) return;

    const allAccepted = change.hunks.every((h) => h.accepted);
    const allRejected = change.hunks.every((h) => !h.accepted);

    if (allAccepted && change.status !== "accepted") {
      await coderService.acceptChange(change.id);
    } else if (allRejected && change.status !== "rejected") {
      await coderService.rejectChange(change.id);
    }
  }

  async function advanceAfterReview(changeId: string, hunkIndex: number) {
    const change = changes.find((c) => c.id === changeId);
    if (change) {
      await finalizeChangeIfReviewed(change);
    }

    await refresh();

    const next = findNextUnreviewed(changeId, hunkIndex);
    if (next) {
      focusChangeId = next.changeId;
      focusHunkIndex = next.hunkIndex;
    } else {
      focusChangeId = null;
      focusHunkIndex = null;
    }
    await scrollToFocus();
  }

  function markReviewed(changeId: string, hunkIndex: number) {
    reviewedHunks = new Set(reviewedHunks).add(hunkKey(changeId, hunkIndex));
  }

  function markAllReviewed(change: FileChange) {
    const next = new Set(reviewedHunks);
    for (const h of change.hunks) {
      next.add(hunkKey(change.id, h.index));
    }
    reviewedHunks = next;
  }

  function display(lines: string[]): string {
    return lines.map((l) => l.replace(/\n$/, "")).join("\n");
  }

  async function acceptHunk(c: FileChange, h: Hunk) {
    if (!h.accepted) {
      await coderService.setHunk(c.id, h.index, true);
    }
    markReviewed(c.id, h.index);
    await advanceAfterReview(c.id, h.index);
  }

  async function rejectHunk(c: FileChange, h: Hunk) {
    if (h.accepted) {
      await coderService.setHunk(c.id, h.index, false);
    }
    markReviewed(c.id, h.index);
    await advanceAfterReview(c.id, h.index);
  }

  async function acceptAllFiles() {
    const list = [...pendingChanges];
    for (const c of list) {
      await coderService.acceptChange(c.id);
      markAllReviewed(c);
    }
    await refresh();
  }

  async function rejectAllFiles() {
    const list = [...pendingChanges];
    for (const c of list) {
      await coderService.rejectChange(c.id);
      markAllReviewed(c);
    }
    await refresh();
  }

  async function acceptAll(c: FileChange) {
    await coderService.acceptChange(c.id);
    markAllReviewed(c);
    await advanceAfterReview(c.id, c.hunks.at(-1)?.index ?? 0);
  }

  async function rejectAll(c: FileChange) {
    await coderService.rejectChange(c.id);
    markAllReviewed(c);
    await advanceAfterReview(c.id, c.hunks.at(-1)?.index ?? 0);
  }

  function startEdit(c: FileChange) {
    editing = c.id;
    draft = c.original_after;
  }

  async function saveEdit(c: FileChange) {
    await coderService.modifyChange(c.id, draft);
    editing = null;
    reviewedHunks = new Set();
    await refresh();
  }

  function goToPrevHunk() {
    if (!focusChangeId || focusHunkIndex === null || !focusPosition) return;
    const { changeIdx, hunkIdx } = focusPosition;
    if (hunkIdx > 0) {
      focusHunkIndex = pendingChanges[changeIdx].hunks[hunkIdx - 1].index;
    } else if (changeIdx > 0) {
      const prev = pendingChanges[changeIdx - 1];
      focusChangeId = prev.id;
      focusHunkIndex = prev.hunks.at(-1)?.index ?? null;
    }
    scrollToFocus();
  }

  function goToNextHunk() {
    if (!focusChangeId || focusHunkIndex === null || !focusPosition) return;
    const { changeIdx, hunkIdx, totalHunks } = focusPosition;
    if (hunkIdx < totalHunks - 1) {
      focusHunkIndex = pendingChanges[changeIdx].hunks[hunkIdx + 1].index;
    } else if (changeIdx < pendingChanges.length - 1) {
      const next = pendingChanges[changeIdx + 1];
      focusChangeId = next.id;
      focusHunkIndex = next.hunks[0]?.index ?? null;
    }
    scrollToFocus();
  }

  $effect(() => {
    changes;
    syncFocus();
  });

  $effect(() => {
    if (focusChangeId && focusHunkIndex !== null) {
      scrollToFocus();
    }
  });
</script>

<div class="space-y-3">
  {#if changes.length === 0}
    <div class="text-xs text-muted-foreground">No file changes yet.</div>
  {:else if pendingChanges.length === 0}
    <div class="rounded-md border border-border px-3 py-4 text-center text-xs text-muted-foreground">
      All changes reviewed.
    </div>
  {/if}

  {#if pendingChanges.length > 0}
    <div
      class="sticky top-0 z-10 flex flex-col gap-2 rounded-md border border-border bg-background px-3 py-2 sm:flex-row sm:items-center sm:justify-between"
    >
      <div class="min-w-0 truncate text-xs text-muted-foreground">
        {#if focusChange && focusPosition}
          <span class="font-medium text-foreground">{focusChange.path}</span>
          <span class="mx-1">·</span>
          hunk {focusPosition.hunkIdx + 1} of {focusPosition.totalHunks}
          <span class="mx-1">·</span>
          file {focusPosition.changeIdx + 1} of {focusPosition.totalChanges}
        {:else}
          {pendingChanges.length} file{pendingChanges.length === 1 ? "" : "s"} to review
        {/if}
      </div>
      <div class="flex shrink-0 flex-wrap items-center justify-end gap-1">
        {#if focusPosition}
          <Button
            size="sm"
            variant="ghost"
            class="h-7 w-7"
            title="Previous hunk"
            onclick={goToPrevHunk}
            disabled={focusPosition.changeIdx === 0 && focusPosition.hunkIdx === 0}
          >
            <ChevronUp class="h-3.5 w-3.5" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 w-7"
            title="Next hunk"
            onclick={goToNextHunk}
            disabled={focusPosition.changeIdx === focusPosition.totalChanges - 1 &&
              focusPosition.hunkIdx === focusPosition.totalHunks - 1}
          >
            <ChevronDown class="h-3.5 w-3.5" />
          </Button>
          <div class="mx-1 h-4 w-px bg-border"></div>
        {/if}
        <Button size="sm" variant="secondary" class="h-7" onclick={acceptAllFiles}>
          <Check class="mr-1 h-3.5 w-3.5" /> Accept all
        </Button>
        <Button size="sm" variant="destructive" class="h-7" onclick={rejectAllFiles}>
          <X class="mr-1 h-3.5 w-3.5" /> Reject all
        </Button>
      </div>
    </div>
  {/if}

  {#each pendingChanges as c (c.id)}
    {@const isPending = c.status === "pending"}
    <div
      class="rounded-md border border-border transition-opacity {isPending
        ? ''
        : 'opacity-60'} {focusChangeId === c.id ? 'ring-1 ring-primary/40' : ''}"
    >
      <div class="divider-edge-b divider-edge-full flex flex-wrap items-center gap-2 px-3 py-2">
        <FileEdit class="h-4 w-4 text-muted-foreground" />
        <span class="font-mono text-xs">{c.path}</span>
        {#if c.created}
          <Badge variant="secondary" class="text-[10px]">new</Badge>
        {/if}
        <Badge
          variant={c.status === "accepted"
            ? "secondary"
            : c.status === "rejected"
              ? "destructive"
              : "outline"}
          class="text-[10px]"
        >
          {c.status}
        </Badge>

        {#if isPending}
          <div class="ml-auto flex items-center gap-1">
            <Button size="sm" variant="ghost" title="Modify" onclick={() => startEdit(c)}>
              <Pencil class="h-3.5 w-3.5" />
            </Button>
            <Button size="sm" variant="secondary" onclick={() => acceptAll(c)}>
              <Check class="mr-1 h-3.5 w-3.5" /> Accept all
            </Button>
            <Button size="sm" variant="destructive" onclick={() => rejectAll(c)}>
              <X class="mr-1 h-3.5 w-3.5" /> Reject all
            </Button>
          </div>
        {/if}
      </div>

      {#if editing === c.id}
        <div class="space-y-2 p-3">
          <Textarea
            bind:value={draft}
            rows={12}
            class="font-mono text-xs"
          />
          <div class="flex gap-2">
            <Button size="sm" onclick={() => saveEdit(c)}>Save to disk</Button>
            <Button size="sm" variant="ghost" onclick={() => (editing = null)}>Cancel</Button>
          </div>
        </div>
      {:else}
        <div class="divide-y divide-border">
          {#each c.hunks as h (h.index)}
            {@const reviewed = isReviewed(c.id, h.index)}
            {@const focused = isFocused(c.id, h.index)}
            <div
              data-hunk-id="{c.id}:{h.index}"
              role="button"
              tabindex="0"
              class="p-2 transition-colors {focused
                ? 'bg-primary/5 ring-2 ring-inset ring-primary/30'
                : reviewed
                  ? 'opacity-70'
                  : ''}"
              onclick={() => {
                if (isPending) {
                  focusChangeId = c.id;
                  focusHunkIndex = h.index;
                }
              }}
              onkeydown={(e) => {
                if (isPending && (e.key === "Enter" || e.key === " ")) {
                  e.preventDefault();
                  focusChangeId = c.id;
                  focusHunkIndex = h.index;
                }
              }}
            >
              <div class="mb-1 flex items-center justify-between gap-2">
                <div class="flex items-center gap-2">
                  <span class="text-[10px] text-muted-foreground">
                    hunk {h.index + 1} · line {h.before_start + 1}
                  </span>
                  {#if reviewed}
                    <Badge
                      variant={h.accepted ? "secondary" : "destructive"}
                      class="text-[10px]"
                    >
                      {h.accepted ? "accepted" : "rejected"}
                    </Badge>
                  {/if}
                </div>

                {#if isPending}
                  <div class="flex items-center gap-1">
                    <Button
                      size="sm"
                      variant={focused && !reviewed ? "secondary" : "ghost"}
                      class="h-6 text-[10px]"
                      onclick={(e) => {
                        e.stopPropagation();
                        acceptHunk(c, h);
                      }}
                    >
                      <Check class="mr-1 h-3 w-3" />
                      {reviewed ? "Accept" : "Accept hunk"}
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      class="h-6 text-[10px] text-destructive hover:text-destructive"
                      onclick={(e) => {
                        e.stopPropagation();
                        rejectHunk(c, h);
                      }}
                    >
                      <X class="mr-1 h-3 w-3" />
                      {reviewed ? "Reject" : "Reject hunk"}
                    </Button>
                  </div>
                {/if}
              </div>
              {#if h.before_lines.length}
                <pre
                  class="overflow-auto rounded bg-red-500/10 px-2 py-1 text-xs {h.accepted
                    ? ''
                    : 'opacity-50'}"><code>{display(h.before_lines).replace(/^/gm, "- ")}</code></pre>
              {/if}
              {#if h.after_lines.length}
                <pre
                  class="overflow-auto rounded bg-green-500/10 px-2 py-1 text-xs {h.accepted
                    ? ''
                    : 'opacity-50 line-through'}"><code
                    >{display(h.after_lines).replace(/^/gm, "+ ")}</code
                  ></pre>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>
