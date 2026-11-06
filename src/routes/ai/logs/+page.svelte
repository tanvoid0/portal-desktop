<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { PageHeader, PageLoading, PageError } from '$lib/components/shell';
  import AILogsViewer from '$lib/domains/ai/components/logs/AILogsViewer.svelte';
  import { toast } from '$lib/utils/toast';
  import { aiLogService } from '$lib/domains/ai';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import type { AILog, LogFilters } from '$lib/domains/ai/types/index.js';

  let filters = $state<LogFilters>({});

  // Keyed on filters: a slow response for an old filter can't overwrite a newer one.
  const logsQuery = createQuery(() => {
    const f = $state.snapshot(filters);
    return {
      queryKey: queryKeys.ai.logs(f),
      queryFn: () =>
        f.search_query ? aiLogService.searchLogs(f.search_query, f) : aiLogService.getLogs(f),
    };
  });
  const logs = $derived<AILog[]>(logsQuery.data ?? []);

  async function handleExport() {
    try {
      // The backend returns the JSON body, not a file path.
      const json = await aiLogService.exportLogs($state.snapshot(filters));
      const url = URL.createObjectURL(new Blob([json], { type: 'application/json' }));
      const a = document.createElement('a');
      a.href = url;
      a.download = `ai-logs-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success(`Exported ${(JSON.parse(json) as unknown[]).length} logs`);
    } catch (error) {
      toast.error('Failed to export logs', error);
    }
  }
</script>

<div class="h-full overflow-y-auto p-4 md:p-6">
  <div class="space-y-4">
    <PageHeader
      title="AI Logs"
      description="View all AI interactions, requests, responses, and errors"
    />
    {#if logsQuery.isPending}
      <PageLoading message="Loading logs..." />
    {:else if logsQuery.isError}
      <PageError
        title="Failed to load logs"
        message={String(logsQuery.error)}
        onRetry={() => logsQuery.refetch()}
      />
    {:else}
      <AILogsViewer {logs} bind:filters onExport={handleExport} />
    {/if}
  </div>
</div>
