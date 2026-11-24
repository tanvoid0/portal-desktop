<script lang="ts">
  import UnifiedLogViewer from '$lib/components/logs/UnifiedLogViewer.svelte';
  import { PageHeader } from '$lib/components/shell';
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="px-3 py-3">
    <PageHeader title="Logs" description="Frontend and backend log output" />
  </div>
  <div class="min-h-0 flex-1">
    <UnifiedLogViewer />
  </div>
</div>
