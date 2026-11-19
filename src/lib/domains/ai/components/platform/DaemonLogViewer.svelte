<!--
  Live tail of the daemon's /system/logs. Polls with the `next` cursor so each
  request returns only new lines; pausing stops the poll, not the server. Lines
  arrive as strings — JSON objects are parsed for a level + message, anything
  else renders raw.
-->
<script lang="ts" module>
  const POLL_MS = 2000;
  const MAX_LINES = 1000;

  interface ParsedLine {
    level?: string;
    message: string;
    raw: string;
  }

  function parseLine(raw: string): ParsedLine {
    try {
      const o: unknown = JSON.parse(raw);
      if (o && typeof o === 'object') {
        const rec = o as Record<string, unknown>;
        const level = typeof rec.level === 'string' ? rec.level : undefined;
        const message =
          typeof rec.message === 'string'
            ? rec.message
            : typeof rec.msg === 'string'
              ? rec.msg
              : raw;
        return { level, message, raw };
      }
    } catch {
      // plain-text line
    }
    return { message: raw, raw };
  }

  function levelClass(level?: string): string {
    switch (level?.toUpperCase()) {
      case 'ERROR':
      case 'CRITICAL':
      case 'FATAL':
        return 'text-red-500';
      case 'WARN':
      case 'WARNING':
        return 'text-amber-500';
      case 'INFO':
        return 'text-sky-500';
      case 'DEBUG':
      case 'TRACE':
        return 'text-muted-foreground';
      default:
        return 'text-foreground';
    }
  }
</script>

<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { toast } from '$lib/utils/toast';
  import { aiPlatform } from '../../state/aiPlatform.svelte.js';
  import { platformSystemService } from '../../services/platformSystemService.js';
  import Pause from '~icons/fa6-solid/pause';
  import Play from '~icons/fa6-solid/play';
  import Trash from '~icons/fa6-solid/trash';
  import CopyIcon from '~icons/fa6-solid/copy';
  import AnglesDown from '~icons/fa6-solid/angles-down';

  let entries = $state<{ id: number; parsed: ParsedLine }[]>([]);
  let cursor = $state(0);
  let paused = $state(false);
  let autoscroll = $state(true);
  let dropped = $state(0);
  let error = $state<string | null>(null);
  let scroller = $state<HTMLDivElement | null>(null);

  let nextId = 0;
  let busy = false;

  async function tick() {
    if (busy) return;
    busy = true;
    try {
      const snap = await platformSystemService.getLogs(cursor);
      if (snap.dropped > 0) dropped += snap.dropped;
      if (snap.lines.length) {
        const added = snap.lines.map((line) => ({ id: nextId++, parsed: parseLine(line) }));
        entries = [...entries, ...added].slice(-MAX_LINES);
      }
      cursor = snap.next;
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  // Poll while not paused and the daemon is usable. Re-runs when either changes.
  $effect(() => {
    if (paused || aiPlatform.blocked) return;
    void tick();
    const id = setInterval(() => void tick(), POLL_MS);
    return () => clearInterval(id);
  });

  // Keep the viewport pinned to the newest line when autoscroll is on.
  $effect(() => {
    entries.length;
    if (!autoscroll || !scroller) return;
    const el = scroller;
    requestAnimationFrame(() => {
      el.scrollTop = el.scrollHeight;
    });
  });

  function clear() {
    entries = [];
    dropped = 0;
  }

  async function copyAll() {
    if (!entries.length) return;
    try {
      await navigator.clipboard.writeText(entries.map((e) => e.parsed.raw).join('\n'));
      toast.success(`Copied ${entries.length} lines`);
    } catch (e) {
      toast.error('Could not copy logs', e);
    }
  }
</script>

<Card class="flex min-h-0 flex-col overflow-hidden">
  <div class="flex flex-wrap items-center gap-2 border-b px-3 py-2">
    <span class="text-sm font-medium">Daemon logs</span>
    {#if dropped > 0}
      <Badge variant="destructive" class="font-normal" title="Lines written faster than polled">
        {dropped} dropped
      </Badge>
    {/if}
    {#if error}
      <span class="truncate text-xs text-destructive" title={error}>{error}</span>
    {/if}
    <div class="ml-auto flex items-center gap-1.5">
      <Button
        variant={autoscroll ? 'secondary' : 'ghost'}
        size="sm"
        class="h-7 gap-1.5 text-xs"
        onclick={() => (autoscroll = !autoscroll)}
      >
        <AnglesDown class="h-3 w-3" />
        Auto-scroll
      </Button>
      <Button
        variant="outline"
        size="sm"
        class="h-7 gap-1.5 text-xs"
        onclick={() => (paused = !paused)}
      >
        {#if paused}<Play class="h-3 w-3" />Resume{:else}<Pause class="h-3 w-3" />Pause{/if}
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        aria-label="Copy all logs"
        disabled={!entries.length}
        onclick={copyAll}
      >
        <CopyIcon class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7"
        aria-label="Clear logs"
        disabled={!entries.length}
        onclick={clear}
      >
        <Trash class="h-3.5 w-3.5" />
      </Button>
    </div>
  </div>

  <div bind:this={scroller} class="h-[420px] overflow-auto bg-muted/30 p-3 font-mono text-xs">
    {#if entries.length === 0}
      <div class="flex h-full items-center justify-center text-muted-foreground">
        {#if aiPlatform.blocked}
          Daemon unavailable — logs resume once it is running.
        {:else if paused}
          Paused.
        {:else}
          Waiting for daemon output…
        {/if}
      </div>
    {:else}
      {#each entries as entry (entry.id)}
        <div class="flex gap-2 py-0.5 leading-relaxed break-all whitespace-pre-wrap">
          {#if entry.parsed.level}
            <span class="w-12 shrink-0 font-semibold {levelClass(entry.parsed.level)}">
              {entry.parsed.level.toUpperCase()}
            </span>
          {/if}
          <span class="min-w-0">{entry.parsed.message}</span>
        </div>
      {/each}
    {/if}
  </div>
</Card>
