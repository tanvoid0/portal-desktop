<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Badge } from "$lib/components/ui/badge";
  import { Label } from "$lib/components/ui/label";
  import {
    ChevronRight,
    File,
    Folder,
    GitBranch,
    GitCommitHorizontal,
    Loader2,
    RefreshCw,
    Sparkles,
  } from "@lucide/svelte";
  import { coderService } from "../services/coderService.js";
  import type { GitCommitDraft } from "../types.js";
  import {
    buildChangeTree,
    flattenChangeTree,
  } from "../utils/changeTree.js";
  import { toast } from "$lib/utils/toast";

  interface Props {
    open?: boolean;
    workspaceRoot: string;
    onCommitted?: () => void;
  }

  let {
    open = $bindable(false),
    workspaceRoot,
    onCommitted,
  }: Props = $props();

  let loading = $state(false);
  let committing = $state(false);
  let error = $state<string | null>(null);
  let draft = $state<GitCommitDraft | null>(null);
  let title = $state("");
  let summary = $state("");
  let expanded = $state<Set<string>>(new Set());

  const tree = $derived(draft ? buildChangeTree(draft.changes) : []);
  const flatRows = $derived(flattenChangeTree(tree, expanded));

  async function prepare(useAi = true) {
    if (!workspaceRoot.trim()) {
      error = "No workspace selected.";
      return;
    }
    loading = true;
    error = null;
    try {
      const result = await coderService.prepareGitCommit(workspaceRoot, useAi);
      draft = result;
      title = result.title;
      summary = result.summary;
      expanded = new Set(
        buildChangeTree(result.changes)
          .filter((n) => !n.isFile)
          .map((n) => n.path),
      );
    } catch (e) {
      error = String(e);
      draft = null;
    } finally {
      loading = false;
    }
  }

  async function commit() {
    if (!title.trim()) {
      toast.error("Commit title is required.");
      return;
    }
    committing = true;
    error = null;
    try {
      const hash = await coderService.gitCommit(
        workspaceRoot,
        title.trim(),
        summary.trim() || null,
      );
      toast.success(`Committed ${hash}`);
      open = false;
      draft = null;
      onCommitted?.();
    } catch (e) {
      error = String(e);
      toast.error(String(e));
    } finally {
      committing = false;
    }
  }

  function toggleFolder(path: string) {
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

  $effect(() => {
    if (open && workspaceRoot) {
      void prepare(true);
    }
    if (!open) {
      draft = null;
      error = null;
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="flex max-h-[85vh] max-w-2xl flex-col gap-0 overflow-hidden p-0">
    <Dialog.Header class="border-b border-border px-6 py-4">
      <Dialog.Title class="flex items-center gap-2 text-lg">
        <GitCommitHorizontal class="h-5 w-5" />
        Commit changes
      </Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground">
        Review working tree changes, edit the message, then commit.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 py-4">
      {#if loading && !draft}
        <div class="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" />
          Reviewing git changes…
        </div>
      {:else if error && !draft}
        <div class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </div>
      {:else if draft}
        <div class="grid gap-3">
          <div class="flex items-center gap-2 text-sm">
            <GitBranch class="h-4 w-4 text-muted-foreground" />
            <span class="text-muted-foreground">Branch</span>
            <code class="rounded bg-muted px-2 py-0.5 font-mono text-xs">
              {draft.branch ?? "detached"}
            </code>
            {#if draft.aiGenerated}
              <Badge variant="secondary" class="gap-1 text-[10px]">
                <Sparkles class="h-3 w-3" />
                AI suggested
              </Badge>
            {/if}
          </div>

          <div class="space-y-1.5">
            <Label for="commit-title" class="text-xs">Commit title</Label>
            <Input
              id="commit-title"
              bind:value={title}
              placeholder="feat: describe the change"
              class="font-mono text-sm"
            />
          </div>

          <div class="space-y-1.5">
            <Label for="commit-summary" class="text-xs">Summary</Label>
            <Textarea
              id="commit-summary"
              bind:value={summary}
              rows={4}
              placeholder="Bullet list of what changed…"
              class="resize-y font-mono text-xs"
            />
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-muted-foreground">
              Changelist ({draft.changes.length} file{draft.changes.length === 1 ? "" : "s"})
            </span>
            <Button
              size="sm"
              variant="ghost"
              class="h-7 gap-1 text-xs"
              onclick={() => void prepare(false)}
              disabled={loading}
            >
              <RefreshCw class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}" />
              Regenerate
            </Button>
          </div>

          <div class="rounded-md border border-border">
            {#if flatRows.length === 0}
              <p class="px-3 py-2 text-xs text-muted-foreground">No files changed.</p>
            {:else}
              {#each flatRows as row (row.node.path + String(row.node.isFile))}
                {@const node = row.node}
                <div
                  class="flex items-center gap-1.5 border-b border-border px-2 py-1.5 text-xs last:border-b-0"
                  style="padding-left: {8 + row.depth * 14}px"
                >
                  {#if !node.isFile}
                    <button
                      type="button"
                      class="flex min-w-0 flex-1 items-center gap-1.5 text-left hover:text-foreground"
                      onclick={() => toggleFolder(node.path)}
                    >
                      <ChevronRight
                        class="h-3.5 w-3.5 shrink-0 transition-transform {expanded.has(node.path)
                          ? 'rotate-90'
                          : ''}"
                      />
                      <Folder class="h-3.5 w-3.5 shrink-0 text-amber-500" />
                      <span class="truncate font-medium">{node.name}</span>
                      <span class="ml-auto shrink-0 font-mono text-[10px] text-muted-foreground">
                        {#if node.additions > 0}
                          <span class="text-green-600 dark:text-green-400">+{node.additions}</span>
                        {/if}
                        {#if node.deletions > 0}
                          <span class="text-red-600 dark:text-red-400">-{node.deletions}</span>
                        {/if}
                      </span>
                    </button>
                  {:else if node.change}
                    <File class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                    <span class="min-w-0 flex-1 truncate font-mono">{node.name}</span>
                    <Badge variant={statusVariant(node.change.status)} class="text-[10px]">
                      {node.change.status}
                    </Badge>
                    {#if node.change.additions > 0}
                      <span class="font-mono text-[10px] text-green-600 dark:text-green-400">
                        +{node.change.additions}
                      </span>
                    {/if}
                    {#if node.change.deletions > 0}
                      <span class="font-mono text-[10px] text-red-600 dark:text-red-400">
                        -{node.change.deletions}
                      </span>
                    {/if}
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        </div>

        {#if error}
          <div class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            {error}
          </div>
        {/if}
      {/if}
    </div>

    <Dialog.Footer class="border-t border-border px-6 py-4">
      <Button variant="outline" onclick={() => (open = false)} disabled={committing}>
        Cancel
      </Button>
      <Button
        onclick={() => void commit()}
        disabled={!draft || !title.trim() || committing || loading}
        class="gap-1.5"
      >
        {#if committing}
          <Loader2 class="h-4 w-4 animate-spin" />
          Committing…
        {:else}
          <GitCommitHorizontal class="h-4 w-4" />
          Commit
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
