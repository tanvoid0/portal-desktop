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
  import Download from '~icons/fa6-solid/download';
  import Trash2 from '~icons/fa6-solid/trash';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import ArrowDown from '~icons/fa6-solid/arrow-down';

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

  // 'all' gets a descriptive label so the closed trigger reads "All levels", not a bare "all".
  const logLevels = [
    { value: 'all', label: 'All levels' },
    ...['DEBUG', 'INFO', 'WARN', 'ERROR'].map((v) => ({ value: v, label: v })),
  ];
  const sources = [
    { value: 'all', label: 'All sources' },
    { value: 'frontend', label: 'frontend' },
    { value: 'backend', label: 'backend' },
  ];
  let contextOptions = $derived([
    { value: 'all', label: 'All contexts' },
    ...availableContexts.map((v) => ({ value: v, label: v })),
  ]);

  let scrollContainer: HTMLElement | null = $state(null);
  let unsubscribe: (() => void) | null = $state(null);
  let isAtBottom = $state(true);
  // Scroll events fire async (next frame). Set right before every programmatic scroll so the
  // resulting `handleScroll` call knows it wasn't the user and skips the auto-scroll-disable
  // check below — otherwise a burst of 2+ log lines lands before that frame, the scroll-position
  // gap reads as "the user scrolled away", and autoScroll flips off with nobody touching anything.
  let suppressScrollCheck = false;

  function scrollToBottom() {
    if (!scrollContainer) return;
    // Only arm the suppression when this write will actually move `scrollTop` — a
    // no-op write (already at the bottom, e.g. a new line hidden by a filter) fires no
    // scroll event, so the flag would never get cleared and would swallow the user's
    // next real scroll.
    if (scrollContainer.scrollTop + scrollContainer.clientHeight < scrollContainer.scrollHeight) {
      suppressScrollCheck = true;
    }
    scrollContainer.scrollTop = scrollContainer.scrollHeight;
  }

  /** The user scrolling away from the bottom on a live stream pauses auto-scroll — it would
   *  otherwise yank them back down on the very next line. Re-checking the box, or the
   *  "Scroll to bottom" button below, both jump back down via the `$effect` and `onclick`. */
  function handleScroll() {
    if (!scrollContainer) return;
    if (suppressScrollCheck) {
      suppressScrollCheck = false;
      isAtBottom = true;
      return;
    }
    const distanceFromBottom =
      scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;
    isAtBottom = distanceFromBottom < 40;
    if (!isAtBottom && autoScroll) autoScroll = false;
  }

  $effect(() => {
    const el = scrollContainer;
    if (!el) return;
    el.addEventListener('scroll', handleScroll, { passive: true });
    return () => el.removeEventListener('scroll', handleScroll);
  });

  // Re-checking Auto-scroll jumps to the bottom immediately, rather than waiting for the
  // next log line (which may never come on a quiet stream).
  $effect(() => {
    if (autoScroll) scrollToBottom();
  });

  // Functions
  function getLevelColor(level: LogLevel): string {
    switch (level) {
      case LogLevel.DEBUG:
        return 'text-cyan-500';
      case LogLevel.INFO:
        return 'text-blue-500';
      case LogLevel.WARN:
        return 'text-yellow-500';
      case LogLevel.ERROR:
        return 'text-red-500';
      default:
        return 'text-gray-500';
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
      setTimeout(scrollToBottom, 10);
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

<div class="flex h-full w-full flex-col">
  <!-- Filters: the only chrome -->
  <div class="flex flex-wrap items-center gap-2 border-b px-3 py-1.5">
    <div class="relative min-w-[200px] flex-1">
      <Search
        class="absolute top-1/2 left-2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
      />
      <Input placeholder="Search logs..." bind:value={searchQuery} class="h-8 pl-8" />
    </div>
    <Select
      bind:value={selectedLevel}
      options={logLevels}
      placeholder="Level"
      aria-label="Filter by level"
      class="w-[130px]"
    />
    <Select
      bind:value={selectedSource}
      options={sources}
      placeholder="Source"
      aria-label="Filter by source"
      class="w-[130px]"
    />
    {#if availableContexts.length > 0}
      <Select
        bind:value={selectedContext}
        options={contextOptions}
        placeholder="Context"
        aria-label="Filter by context"
        class="w-[150px]"
      />
    {/if}
    <div class="flex items-center gap-2 text-sm">
      <Checkbox bind:checked={autoScroll} id="log-auto-scroll" />
      <Label for="log-auto-scroll">Auto-scroll</Label>
    </div>
    <span class="text-sm whitespace-nowrap text-muted-foreground">
      Showing {filteredLogs.length} of {logs.length}
    </span>
    <Button
      variant="ghost"
      size="sm"
      onclick={clearLogs}
      aria-label="Clear logs"
      title="Clear logs"
    >
      <Trash2 class="h-4 w-4" />
    </Button>
    <Button
      variant="ghost"
      size="sm"
      onclick={exportLogs}
      aria-label="Export logs"
      title="Export logs"
    >
      <Download class="h-4 w-4" />
    </Button>
  </div>

  <!-- Logs Display -->
  <div class="relative min-h-0 flex-1">
    <ScrollArea class="h-full" bind:viewportRef={scrollContainer}>
      {#if filteredLogs.length === 0}
        <div class="py-8 text-center text-muted-foreground">No logs found</div>
      {:else}
        {#each filteredLogs as entry, i (i)}
          <div
            class="flex items-baseline gap-2 px-3 py-0.5 font-mono text-xs leading-5 hover:bg-muted/50"
          >
            <span
              class={'w-3 shrink-0 text-center font-bold ' + getLevelColor(entry.level)}
              title={LogLevel[entry.level]}
            >
              <span aria-hidden="true">{LogLevel[entry.level][0]}</span>
              <span class="sr-only">{LogLevel[entry.level]}</span>
            </span>
            <span class="shrink-0 text-muted-foreground">{formatTimestamp(entry.timestamp)}</span>
            <Badge variant="outline" class="px-1 py-0 text-xs">{entry.source}</Badge>
            {#if entry.context}
              <Badge variant="outline" class="px-1 py-0 text-xs">{entry.context}</Badge>
            {/if}
            <span class="min-w-0 flex-1 break-words">{entry.message}</span>
          </div>
        {/each}
      {/if}
    </ScrollArea>
    {#if !isAtBottom}
      <Button
        variant="secondary"
        size="sm"
        class="absolute right-3 bottom-3 shadow-md"
        aria-label="Scroll to bottom"
        onclick={() => {
          autoScroll = true;
          scrollToBottom();
          isAtBottom = true;
        }}
      >
        <ArrowDown class="mr-1 h-3 w-3" />
        Scroll to bottom
      </Button>
    {/if}
  </div>
</div>
