<script lang="ts">
  import UnifiedLogViewer from '$lib/components/logs/UnifiedLogViewer.svelte';
</script>

<div class="h-full w-full p-6">
  <div class="space-y-4">
    <div>
      <h1 class="text-2xl font-bold">Application Logs</h1>
      <p class="text-muted-foreground">View all frontend and backend logs in real-time</p>
    </div>
    <div class="h-[calc(100vh-12rem)]">
      <UnifiedLogViewer />
    </div>
  </div>
</div>
