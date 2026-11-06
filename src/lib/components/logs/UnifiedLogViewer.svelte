<!--
	Unified Log Viewer - Displays both frontend and backend logs
	Features: filtering, search, export, real-time updates
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    logger,
    type LogEntry,
    LogLevel,
    type LogSource,
  } from '$lib/domains/shared/services/logger';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import Select from '$lib/components/ui/select.svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import Download from '~icons/fa6-solid/download';
  import Trash2 from '~icons/fa6-solid/trash';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Filter from '~icons/fa6-solid/filter';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import Info from '~icons/fa6-solid/circle-info';
  import AlertTriangle from '~icons/fa6-solid/triangle-exclamation';
  import X from '~icons/fa6-solid/xmark';
  import Bug from '~icons/fa6-solid/bug';

  // State
  let logs = $state<LogEntry[]>([]);
  let searchQuery = $state('');
  let selectedLevel = $state<string>('all');
  let selectedSource = $state<string>('all');
  let selectedContext = $state<string>('all');
  let autoScroll = $state(true);

  // Derived state
  let filteredLogs = $derived.by(() => {
    // logger is not reactive: `logs` is the signal that it changed.
    void logs;
    return logger.getFilteredEntries({
      level: selectedLevel !== 'all' ? LogLevel[selectedLevel as keyof typeof LogLevel] : undefined,
      source: selectedSource !== 'all' ? (selectedSource as LogSource) : undefined,
      context: selectedContext !== 'all' ? selectedContext : undefined,
      search: searchQuery || undefined,
    });
  });

  let availableContexts = $derived.by(() => {
    void logs;
    const contexts = new Set<string>();
    logger.getEntries().forEach((entry) => {
      if (entry.context) {
        contexts.add(entry.context);
      }
    });
    return Array.from(contexts).sort();
  });

  const logLevels = ['all', 'DEBUG', 'INFO', 'WARN', 'ERROR'];
  const sources = ['all', 'frontend', 'backend'];

  let scrollContainer: HTMLElement | null = $state(null);
  let unsubscribe: (() => void) | null = $state(null);

  // Functions
  function getLevelColor(level: LogLevel): string {
    switch (level) {
      case LogLevel.DEBUG:
        return 'bg-cyan-500/10 text-cyan-500 border-cyan-500/20';
      case LogLevel.INFO:
        return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
      case LogLevel.WARN:
        return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
      case LogLevel.ERROR:
        return 'bg-red-500/10 text-red-500 border-red-500/20';
      default:
        return 'bg-gray-500/10 text-gray-500 border-gray-500/20';
    }
  }

  function getLevelIcon(level: LogLevel) {
    switch (level) {
      case LogLevel.DEBUG:
        return Bug;
      case LogLevel.INFO:
        return Info;
      case LogLevel.WARN:
        return AlertTriangle;
      case LogLevel.ERROR:
        return AlertCircle;
      default:
        return Info;
    }
  }

  function formatTimestamp(date: Date): string {
    return date.toISOString().replace('T', ' ').substring(0, 23);
  }

  function clearLogs() {
    logger.clear();
    updateLogs();
  }

  function exportLogs() {
    const logText = filteredLogs
      .map((entry) => {
        const timestamp = formatTimestamp(entry.timestamp);
        const level = LogLevel[entry.level];
        const source = entry.source.toUpperCase();
        const context = entry.context ? `[${entry.context}]` : '';
        return `[${timestamp}] [${source}] [${level}]${context} ${entry.message}`;
      })
      .join('\n');

    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logs-${new Date().toISOString().split('T')[0]}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function updateLogs() {
    logs = logger.getEntries();
    if (autoScroll && scrollContainer) {
      setTimeout(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      }, 10);
    }
  }

  function scrollToBottom() {
    if (scrollContainer) {
      scrollContainer.scrollTo({ top: scrollContainer.scrollHeight, behavior: 'smooth' });
    }
  }

  // Lifecycle
  onMount(() => {
    updateLogs();
    unsubscribe = logger.onLog(() => {
      updateLogs();
    });
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });
</script>

<Card class="flex h-full w-full flex-col">
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle class="flex items-center gap-2">
        <Filter class="h-5 w-5" />
        Unified Logs
      </CardTitle>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" onclick={clearLogs}>
          <Trash2 class="mr-2 h-4 w-4" />
          Clear
        </Button>
        <Button variant="outline" size="sm" onclick={exportLogs}>
          <Download class="mr-2 h-4 w-4" />
          Export
        </Button>
      </div>
    </div>
  </CardHeader>
  <CardContent class="flex min-h-0 flex-1 flex-col gap-4">
    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-2">
      <div class="min-w-[200px] flex-1">
        <div class="relative">
          <Search
            class="absolute top-1/2 left-2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
          />
          <Input placeholder="Search logs..." bind:value={searchQuery} class="pl-8" />
        </div>
      </div>
      <Select
        bind:value={selectedLevel}
        options={logLevels}
        placeholder="Level"
        aria-label="Filter by level"
        class="w-[120px]"
      />
      <Select
        bind:value={selectedSource}
        options={sources}
        placeholder="Source"
        aria-label="Filter by source"
        class="w-[120px]"
      />
      {#if availableContexts.length > 0}
        <Select
          bind:value={selectedContext}
          options={['all', ...availableContexts]}
          placeholder="Context"
          aria-label="Filter by context"
          class="w-[150px]"
        />
      {/if}
      <div class="flex items-center gap-2 text-sm">
        <Checkbox bind:checked={autoScroll} id="log-auto-scroll" />
        <Label for="log-auto-scroll">Auto-scroll</Label>
      </div>
    </div>

    <!-- Logs Display -->
    <ScrollArea class="flex-1 rounded-md border" bind:viewportRef={scrollContainer}>
      <div class="space-y-1 p-4">
        {#if filteredLogs.length === 0}
          <div class="py-8 text-center text-muted-foreground">No logs found</div>
        {:else}
          {#each filteredLogs as entry (entry.timestamp.getTime() + entry.message)}
            {@const LevelIcon = getLevelIcon(entry.level)}
            <div class="flex items-start gap-2 rounded p-2 text-sm hover:bg-muted/50">
              <Badge variant="outline" class={getLevelColor(entry.level) + ' text-xs'}>
                <LevelIcon class="mr-1 h-3 w-3" />
                {LogLevel[entry.level]}
              </Badge>
              <Badge variant="outline" class="text-xs">
                {entry.source}
              </Badge>
              {#if entry.context}
                <Badge variant="outline" class="text-xs">
                  {entry.context}
                </Badge>
              {/if}
              <span class="font-mono text-xs text-muted-foreground">
                {formatTimestamp(entry.timestamp)}
              </span>
              <span class="flex-1">{entry.message}</span>
            </div>
          {/each}
        {/if}
      </div>
    </ScrollArea>

    <!-- Stats -->
    <div class="flex items-center justify-between text-sm text-muted-foreground">
      <span>Showing {filteredLogs.length} of {logs.length} logs</span>
      <Button variant="ghost" size="sm" onclick={scrollToBottom}>Scroll to bottom</Button>
    </div>
  </CardContent>
</Card>
