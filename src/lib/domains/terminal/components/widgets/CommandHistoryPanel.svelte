<script lang="ts">
  import CommandHistory from "../CommandHistory.svelte";

  interface Props {
    tabId: string;
  }

  let { tabId }: Props = $props();
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="border-b border-border p-2">
    <div class="text-sm font-medium text-foreground">Command History</div>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto">
    <CommandHistory {tabId} />
  </div>
</div>
