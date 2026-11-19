<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Badge } from '$lib/components/ui/badge';
  import Check from '~icons/fa6-solid/check';
  import X from '~icons/fa6-solid/xmark';
  import FileEdit from '~icons/fa6-solid/file-pen';
  import Pencil from '~icons/fa6-solid/pencil';
  import { ChangesReviewState } from '../state/changesReview.svelte.js';
  import type { FileChange } from '../types.js';
  import ChangesReviewBar from './ChangesReviewBar.svelte';
  import ChangeHunk from './ChangeHunk.svelte';

  interface Props {
    changes: FileChange[];
    onRefresh: () => void | Promise<void>;
  }

  let { changes, onRefresh }: Props = $props();

  const r = new ChangesReviewState(
    () => changes,
    () => onRefresh()
  );
</script>

<div class="space-y-3">
  {#if changes.length === 0}
    <div class="text-xs text-muted-foreground">No file changes yet.</div>
  {:else if r.pendingChanges.length === 0}
    <div
      class="rounded-md border border-border px-3 py-4 text-center text-xs text-muted-foreground"
    >
      All changes reviewed.
    </div>
  {/if}

  {#if r.pendingChanges.length > 0}
    <ChangesReviewBar
      focusPath={r.focusChange?.path}
      position={r.focusPosition}
      pendingCount={r.pendingChanges.length}
      onPrev={r.goToPrevHunk}
      onNext={r.goToNextHunk}
      onAcceptAll={r.acceptAllFiles}
      onRejectAll={r.rejectAllFiles}
    />
  {/if}

  {#each r.pendingChanges as c (c.id)}
    {@const isPending = c.status === 'pending'}
    <div
      class="rounded-md border border-border transition-opacity {isPending
        ? ''
        : 'opacity-60'} {r.focusChangeId === c.id ? 'ring-1 ring-primary/40' : ''}"
    >
      <div class="divider-edge-b divider-edge-full flex flex-wrap items-center gap-2 px-3 py-2">
        <FileEdit class="h-4 w-4 text-muted-foreground" />
        <span class="font-mono text-xs">{c.path}</span>
        {#if c.created}
          <Badge variant="secondary" class="text-xs">new</Badge>
        {/if}
        <Badge
          variant={c.status === 'accepted'
            ? 'secondary'
            : c.status === 'rejected'
              ? 'destructive'
              : 'outline'}
          class="text-xs"
        >
          {c.status}
        </Badge>

        {#if isPending}
          <div class="ml-auto flex items-center gap-1">
            <Button size="sm" variant="ghost" title="Modify" onclick={() => r.startEdit(c)}>
              <Pencil class="h-3.5 w-3.5" />
            </Button>
            <Button size="sm" variant="secondary" onclick={() => r.acceptAll(c)}>
              <Check class="mr-1 h-3.5 w-3.5" /> Accept all
            </Button>
            <Button size="sm" variant="destructive" onclick={() => r.rejectAll(c)}>
              <X class="mr-1 h-3.5 w-3.5" /> Reject all
            </Button>
          </div>
        {/if}
      </div>

      {#if r.editing === c.id}
        <div class="space-y-2 p-3">
          <Textarea bind:value={r.draft} rows={12} class="font-mono text-xs" />
          <div class="flex gap-2">
            <Button size="sm" onclick={() => r.saveEdit(c)}>Save to disk</Button>
            <Button size="sm" variant="ghost" onclick={() => (r.editing = null)}>Cancel</Button>
          </div>
        </div>
      {:else}
        <div class="divide-y divide-border">
          {#each c.hunks as h (h.index)}
            <ChangeHunk
              changeId={c.id}
              hunk={h}
              {isPending}
              reviewed={r.isReviewed(c.id, h.index)}
              focused={r.isFocused(c.id, h.index)}
              onFocus={() => {
                r.focus(c.id, h.index);
              }}
              onAccept={() => r.acceptHunk(c, h)}
              onReject={() => r.rejectHunk(c, h)}
            />
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>
