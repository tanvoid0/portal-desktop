<script lang="ts">
  import FaDownload from '~icons/fa6-solid/download';
  import FaFileLines from '~icons/fa6-solid/file-lines';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import LogEntry from './LogEntry.svelte';
  import LogFiltersComponent from './LogFilters.svelte';
  import { Button } from '$lib/components/ui/button';
  import type { AILog, LogFilters } from '../../types/index.js';

  interface Props {
    logs: AILog[];
    filters?: LogFilters;
    onFiltersChange?: (filters: LogFilters) => void;
    onExport?: () => void;
  }

  let {
    logs = $bindable<AILog[]>([]),
    filters = $bindable<LogFilters>({}),
    onFiltersChange,
    onExport,
  }: Props = $props();

  function handleClearFilters() {
    filters = {};
    onFiltersChange?.(filters);
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <LogFiltersComponent bind:filters {onFiltersChange} onClear={handleClearFilters} />
    {#if onExport}
      <Button variant="outline" onclick={onExport}>
        <FaDownload class="mr-2 h-4 w-4" />
        Export
      </Button>
    {/if}
  </div>
  <ScrollArea class="h-[calc(100vh-300px)]">
    <div class="space-y-2">
      {#if logs.length === 0}
        <div class="py-8 text-center text-muted-foreground">
          <FaFileLines class="mx-auto mb-2 h-12 w-12 opacity-50" />
          <p class="text-sm">No logs found</p>
        </div>
      {:else}
        {#each logs as log}
          <LogEntry {log} />
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
