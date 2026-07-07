<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import CoderSessionCard from "./CoderSessionCard.svelte";
  import { Plus, Bot } from "@lucide/svelte";
  import type { CoderThread } from "../types.js";

  interface Props {
    threads: CoderThread[];
    onThreadClick?: (thread: CoderThread) => void;
    onCreateNew?: () => void;
    onDeleteThread?: (thread: CoderThread) => void;
    selectedThreadId?: string | null;
    runningThreadIds?: Set<string>;
    queuedCountFor?: (threadId: string) => number;
  }

  let {
    threads = $bindable<CoderThread[]>([]),
    onThreadClick,
    onCreateNew,
    onDeleteThread,
    selectedThreadId,
    runningThreadIds = new Set<string>(),
    queuedCountFor,
  }: Props = $props();

  let searchQuery = $state("");
  let filteredThreads = $derived(
    threads.filter(
      (t) =>
        t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        t.workspace_root.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center gap-2 border-b p-2.5">
    <Input
      placeholder="Search sessions..."
      bind:value={searchQuery}
      class="h-8 flex-1 text-sm"
    />
    <Button onclick={onCreateNew} size="sm" class="h-8" title="New session">
      <Plus class="h-3.5 w-3.5" />
    </Button>
  </div>
  <ScrollArea class="flex-1">
    <div class="space-y-2 p-2">
      {#if filteredThreads.length === 0}
        <div class="py-6 text-center text-muted-foreground">
          <Bot class="mx-auto mb-1.5 h-8 w-8 opacity-50" />
          <p class="text-xs">
            {searchQuery ? "No sessions found" : "No sessions yet"}
          </p>
        </div>
      {:else}
        {#each filteredThreads as t}
          <CoderSessionCard
            thread={t}
            onClick={() => onThreadClick?.(t)}
            onDelete={onDeleteThread}
            isActive={selectedThreadId === t.id}
            isRunning={runningThreadIds.has(t.id)}
            queuedCount={queuedCountFor?.(t.id) ?? 0}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
