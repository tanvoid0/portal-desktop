<!-- Sticky review bar: where the focused hunk is, prev/next, and accept/reject-all for every pending file. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Check from '~icons/fa6-solid/check';
  import X from '~icons/fa6-solid/xmark';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import ChevronUp from '~icons/fa6-solid/chevron-up';

  interface Props {
    focusPath?: string;
    position: {
      changeIdx: number;
      hunkIdx: number;
      totalChanges: number;
      totalHunks: number;
    } | null;
    pendingCount: number;
    onPrev: () => void;
    onNext: () => void;
    onAcceptAll: () => void;
    onRejectAll: () => void;
  }

  let { focusPath, position, pendingCount, onPrev, onNext, onAcceptAll, onRejectAll }: Props =
    $props();
</script>

<div
  class="sticky top-0 z-10 flex flex-col gap-2 rounded-md border border-border bg-background px-3 py-2 sm:flex-row sm:items-center sm:justify-between"
>
  <div class="min-w-0 truncate text-xs text-muted-foreground">
    {#if focusPath && position}
      <span class="font-medium text-foreground">{focusPath}</span>
      <span class="mx-1">·</span>
      hunk {position.hunkIdx + 1} of {position.totalHunks}
      <span class="mx-1">·</span>
      file {position.changeIdx + 1} of {position.totalChanges}
    {:else}
      {pendingCount} file{pendingCount === 1 ? '' : 's'} to review
    {/if}
  </div>
  <div class="flex shrink-0 flex-wrap items-center justify-end gap-1">
    {#if position}
      <Button
        size="sm"
        variant="ghost"
        class="h-7 w-7"
        title="Previous hunk"
        onclick={onPrev}
        disabled={position.changeIdx === 0 && position.hunkIdx === 0}
      >
        <ChevronUp class="h-3.5 w-3.5" />
      </Button>
      <Button
        size="sm"
        variant="ghost"
        class="h-7 w-7"
        title="Next hunk"
        onclick={onNext}
        disabled={position.changeIdx === position.totalChanges - 1 &&
          position.hunkIdx === position.totalHunks - 1}
      >
        <ChevronDown class="h-3.5 w-3.5" />
      </Button>
      <div class="mx-1 h-4 w-px bg-border"></div>
    {/if}
    <Button size="sm" variant="secondary" class="h-7" onclick={onAcceptAll}>
      <Check class="mr-1 h-3.5 w-3.5" /> Accept all
    </Button>
    <Button size="sm" variant="destructive" class="h-7" onclick={onRejectAll}>
      <X class="mr-1 h-3.5 w-3.5" /> Reject all
    </Button>
  </div>
</div>
