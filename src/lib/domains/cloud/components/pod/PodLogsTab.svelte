<!-- Pod logs: search/filter panel plus parsed log display; loads lazily once its tab is active -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Download from '~icons/fa6-solid/download';
  import Loading from '$lib/components/ui/loading.svelte';
  import { podApi } from '$lib/domains/cloud/api/podApi';
  import { toastActions } from '$lib/utils/toast';
  import { downloadTextFile } from '$lib/utils/downloadTextFile';
  import LogSearchPanel from '$lib/domains/cloud/components/logs/LogSearchPanel.svelte';
  import LogsDisplay from '$lib/domains/cloud/components/logs/LogsDisplay.svelte';
  import { parseRawLogs } from '$lib/domains/cloud/utils/logParser';

  interface Props {
    pod: ICloudResource;
    active: boolean;
  }
  let { pod, active }: Props = $props();

  let logs = $state('');
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  let logSearchQuery = $state('');
  let selectedLogContainer = $state('');
  let selectedLogSeverity = $state('');
  let logTailLines = $state(1000);
  let logViewMode = $state<'detailed' | 'compact' | 'raw'>('detailed');
  const viewModes = [
    ['detailed', 'Detailed'],
    ['compact', 'Compact'],
    ['raw', 'Raw'],
  ] as const;

  const logContainers = $derived.by(() => {
    if (!pod.metadata?.containers) return [];
    const containers = pod.metadata.containers;
    if (Array.isArray(containers)) {
      return containers.map((c: any) => c.name || c).filter(Boolean);
    }
    return [];
  });

  const parsedLogs = $derived.by(() => {
    if (!logs) return [];
    const containerName = selectedLogContainer || logContainers[0] || 'app';
    return parseRawLogs(logs, pod.name, containerName);
  });

  $effect(() => {
    if (active && !logs && !logsLoading && !logsError?.includes('waiting to start')) {
      loadLogs();
    }
  });

  async function loadLogs(container?: string, tailLines?: number) {
    if (logsLoading) return;

    try {
      logsLoading = true;
      logsError = null;

      const containerToUse = container || selectedLogContainer || null;
      const tailLinesToUse = tailLines || logTailLines || 1000;

      // Use the getLogs method from the pod resource if available
      if (pod.getLogs) {
        // Type assertion needed because type definition doesn't match implementation
        const getLogsFn = pod.getLogs as (
          container?: string,
          tailLines?: number
        ) => Promise<string>;
        logs = await getLogsFn(containerToUse || undefined, tailLinesToUse || undefined);
      } else {
        logs = await podApi.getLogs({
          namespace: pod.namespace,
          podName: pod.name,
          container: containerToUse,
          follow: false,
          tailLines: tailLinesToUse,
        });
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      const isPodInitializing =
        errorMessage.includes('PodInitializing') ||
        errorMessage.includes('waiting to start') ||
        errorMessage.includes('is waiting to start');

      if (isPodInitializing) {
        // Expected while the pod starts - no error toast, keep logs empty so we can retry later
        logsError = 'Container is waiting to start. Logs will be available once the pod is ready.';
        logs = '';
      } else {
        logsError = errorMessage;
        console.error('Failed to load logs:', err);
        toastActions.error(`Failed to load logs: ${logsError}`);
      }
    } finally {
      logsLoading = false;
    }
  }

  // Search and severity filter client-side; container and tail lines reload.
  function handleLogContainerChange(container: string) {
    selectedLogContainer = container;
    loadLogs(container, logTailLines);
  }

  function handleLogSeverityChange(severity: string) {
    selectedLogSeverity = severity;
  }

  function handleLogTailLinesChange(lines: number) {
    logTailLines = lines;
    loadLogs(selectedLogContainer || undefined, lines);
  }

  function handleClearLogFilters() {
    logSearchQuery = '';
    selectedLogContainer = '';
    selectedLogSeverity = '';
    logTailLines = 1000;
    loadLogs();
  }
</script>

<LogSearchPanel
  bind:searchQuery={logSearchQuery}
  bind:selectedContainer={selectedLogContainer}
  bind:selectedSeverity={selectedLogSeverity}
  bind:tailLines={logTailLines}
  containers={logContainers}
  onSearchChange={(query) => (logSearchQuery = query)}
  onContainerChange={handleLogContainerChange}
  onSeverityChange={handleLogSeverityChange}
  onTailLinesChange={handleLogTailLinesChange}
  onClearFilters={handleClearLogFilters}
/>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle>
        Pod Logs
        {#if parsedLogs.length > 0}
          <span class="ml-2 text-sm font-normal text-muted-foreground">
            ({parsedLogs.length} entries)
          </span>
        {/if}
      </CardTitle>
      <div class="flex items-center gap-2">
        <div class="flex items-center gap-1 rounded-md border p-1">
          {#each viewModes as [mode, label]}
            <Button
              variant={logViewMode === mode ? 'default' : 'ghost'}
              size="sm"
              class="h-7 px-2 text-xs"
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                logViewMode = mode;
              }}
            >
              {label}
            </Button>
          {/each}
        </div>
        <Button variant="outline" size="sm" onclick={() => loadLogs()} disabled={logsLoading}>
          <RefreshCw class="mr-2 h-4 w-4 {logsLoading ? 'animate-spin' : ''}" />
          Refresh
        </Button>
        {#if logs}
          <Button
            variant="outline"
            size="sm"
            onclick={() => downloadTextFile(logs, `${pod.name}-logs.txt`)}
          >
            <Download class="mr-2 h-4 w-4" />
            Download
          </Button>
        {/if}
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if logsLoading}
      <div class="flex items-center justify-center py-8">
        <Loading text="Loading logs..." />
      </div>
    {:else if logsError}
      <div class="flex flex-col items-center justify-center space-y-4 py-8">
        <div class="text-center text-muted-foreground">{logsError}</div>
        {#if logsError.includes('waiting to start')}
          <Button
            variant="outline"
            size="sm"
            onclick={() => {
              logsError = null;
              loadLogs();
            }}
          >
            <RefreshCw class="mr-2 h-4 w-4" />
            Retry
          </Button>
        {/if}
      </div>
    {:else if parsedLogs.length > 0}
      <LogsDisplay
        logs={parsedLogs}
        searchQuery={logSearchQuery}
        severityFilter={selectedLogSeverity}
        viewMode={logViewMode}
        onFilterBySeverity={handleLogSeverityChange}
      />
    {:else if logs}
      <div class="py-8 text-center text-muted-foreground">
        <p>No logs available to parse</p>
        <p class="mt-2 text-xs">Raw logs: {logs.split('\n').length} lines</p>
      </div>
    {:else}
      <p class="text-muted-foreground">No logs available</p>
    {/if}
  </CardContent>
</Card>
