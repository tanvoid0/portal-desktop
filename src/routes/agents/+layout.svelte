<!-- Agents layout — just a header for now. One route (Run) in P1; grows a tab
     bar like `src/routes/ai/+layout.svelte` once a sessions list/board exist. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet<[]> } = $props();
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div
    class="divider-edge-b divider-edge-full flex shrink-0 items-center gap-2 bg-background px-3 py-2 md:px-4"
  >
    <h1 class="text-sm font-medium">Agents</h1>
  </div>
  <div class="min-h-0 flex-1 overflow-hidden">
    {@render children()}
  </div>
</div>
