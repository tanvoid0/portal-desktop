<!--
	The commit bar: what is ticked, how much that frees, and the one button that
	acts on it. Sticky, because the list it summarises is usually longer than the
	screen and the count is the thing you check before pressing the button.

	The action is always phrased as where the files go, never as "remove" — the
	whole promise of this page is that nothing is destroyed.
-->

<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Trash2 } from '@lucide/svelte';
  import { fmtBytes } from '../../utils';

  let {
    count,
    bytes,
    actionLabel = 'Move to Recycle Bin',
    disabled = false,
    onaction,
    tools,
  }: {
    count: number;
    bytes: number;
    actionLabel?: string;
    disabled?: boolean;
    onaction: () => void;
    /** Per-tab select/expand controls, rendered on the left. */
    tools?: Snippet;
  } = $props();
</script>

<div
  class="sticky top-0 z-10 mb-3 flex flex-wrap items-center gap-2 rounded-lg border border-border bg-background/95 px-3 py-2 backdrop-blur"
>
  {@render tools?.()}
  <div class="ml-auto flex items-center gap-3">
    <span class="text-sm text-muted-foreground">
      {count} selected ·
      <span class="font-semibold tabular-nums text-foreground">{fmtBytes(bytes)}</span>
    </span>
    <Button variant="destructive" size="sm" onclick={onaction} disabled={disabled || count === 0}>
      <Trash2 class="size-4" />
      {actionLabel}
    </Button>
  </div>
</div>
