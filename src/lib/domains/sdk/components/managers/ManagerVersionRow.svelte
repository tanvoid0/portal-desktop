<script lang="ts">
  import type { Snippet } from 'svelte';

  let { version, badge, actions }: { version: string; badge?: Snippet; actions: Snippet } =
    $props();
</script>

<div class="flex items-center justify-between rounded-lg border p-3 hover:bg-muted/50">
  <div class="flex items-center gap-3">
    <span class="font-mono text-sm">{version}</span>
    {@render badge?.()}
  </div>
  <div class="flex items-center gap-2">{@render actions()}</div>
</div>
