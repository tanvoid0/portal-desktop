<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { FileEdit, GitCommitHorizontal, RefreshCw } from "@lucide/svelte";
  import { coderService } from "../services/coderService.js";
  import type { GitFileChange } from "../types.js";

  interface Props {
    workspaceRoot: string;
    onCommit?: () => void;
  }

  let { workspaceRoot, onCommit }: Props = $props();

  let changes = $state<GitFileChange[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let expanded = $state<Set<string>>(new Set());

  async function refresh() {
    if (!workspaceRoot.trim()) {
      changes = [];
      return;
    }
    loading = true;
    error = null;
    try {
      changes = await coderService.listGitChanges(workspaceRoot);
      if (expanded.size === 0 && changes.length > 0) {
        expanded = new Set([changes[0].path]);
      }
    } catch (e) {
      error = String(e);
      changes = [];
    } finally {
      loading = false;
    }
  }

  function toggle(path: string) {
    const next = new Set(expanded);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expanded = next;
  }

  function statusVariant(
    status: string,
  ): "secondary" | "destructive" | "outline" {
    if (status === "deleted") return "destructive";
    if (status === "untracked" || status === "added") return "secondary";
    return "outline";
  }

  onMount(() => {
    void refresh();
  });

  $effect(() => {
    workspaceRoot;
    void refresh();
  });
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between gap-2">
    <p class="text-xs text-muted-foreground">
      Working tree changes vs last commit.
    </p>
    <div class="flex items-center gap-1">
      {#if onCommit && changes.length > 0}
        <Button
          size="sm"
          variant="outline"
          class="h-7 gap-1 text-xs"
          onclick={onCommit}
        >
          <GitCommitHorizontal class="h-3.5 w-3.5" />
          Commit…
        </Button>
      {/if}
      <Button
        size="sm"
        variant="ghost"
        class="h-7 gap-1 text-xs"
        onclick={refresh}
        disabled={loading}
      >
        <RefreshCw class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </div>

  {#if error}
    <div class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
      {error}
    </div>
  {:else if loading && changes.length === 0}
    <div class="text-xs text-muted-foreground">Loading git changes…</div>
  {:else if changes.length === 0}
    <div class="text-xs text-muted-foreground">No git changes in workspace.</div>
  {:else}
    {#each changes as c (c.path)}
      <div class="rounded-md border border-border">
        <button
          type="button"
          class="flex w-full items-center gap-2 border-b border-border px-3 py-2 text-left text-xs hover:bg-muted/40"
          onclick={() => toggle(c.path)}
        >
          <FileEdit class="h-4 w-4 shrink-0 text-muted-foreground" />
          <span class="min-w-0 flex-1 truncate font-mono">{c.path}</span>
          <Badge variant={statusVariant(c.status)} class="text-[10px]">
            {c.status}
          </Badge>
          {#if c.additions > 0}
            <span class="font-mono text-[10px] text-green-600 dark:text-green-400">
              +{c.additions}
            </span>
          {/if}
          {#if c.deletions > 0}
            <span class="font-mono text-[10px] text-red-600 dark:text-red-400">
              -{c.deletions}
            </span>
          {/if}
        </button>
        {#if expanded.has(c.path) && c.diff}
          <pre
            class="max-h-80 overflow-auto p-2 font-mono text-[11px] leading-relaxed"><code>{c.diff}</code></pre>
        {/if}
      </div>
    {/each}
  {/if}
</div>
