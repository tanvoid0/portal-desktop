<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Check, X, FileEdit, Pencil } from "@lucide/svelte";
  import { coderService } from "../services/coderService.js";
  import type { FileChange, Hunk } from "../types.js";

  interface Props {
    changes: FileChange[];
    onRefresh: () => void;
  }

  let { changes, onRefresh }: Props = $props();

  // change id currently being edited manually -> draft content
  let editing = $state<string | null>(null);
  let draft = $state("");

  function display(lines: string[]): string {
    return lines.map((l) => l.replace(/\n$/, "")).join("\n");
  }

  async function acceptAll(c: FileChange) {
    await coderService.acceptChange(c.id);
    onRefresh();
  }
  async function rejectAll(c: FileChange) {
    await coderService.rejectChange(c.id);
    onRefresh();
  }
  async function toggleHunk(c: FileChange, h: Hunk) {
    await coderService.setHunk(c.id, h.index, !h.accepted);
    onRefresh();
  }
  function startEdit(c: FileChange) {
    editing = c.id;
    draft = c.original_after;
  }
  async function saveEdit(c: FileChange) {
    await coderService.modifyChange(c.id, draft);
    editing = null;
    onRefresh();
  }
</script>

<div class="space-y-3">
  {#if changes.length === 0}
    <div class="text-xs text-muted-foreground">No file changes yet.</div>
  {/if}

  {#each changes as c (c.id)}
    <div class="rounded-md border border-border">
      <div class="flex flex-wrap items-center gap-2 border-b border-border px-3 py-2">
        <FileEdit class="h-4 w-4 text-muted-foreground" />
        <span class="font-mono text-xs">{c.path}</span>
        {#if c.created}
          <Badge variant="secondary" class="text-[10px]">new</Badge>
        {/if}
        <Badge
          variant={c.status === "accepted"
            ? "secondary"
            : c.status === "rejected"
              ? "destructive"
              : "outline"}
          class="text-[10px]"
        >
          {c.status}
        </Badge>

        <div class="ml-auto flex items-center gap-1">
          <Button size="sm" variant="ghost" title="Modify" onclick={() => startEdit(c)}>
            <Pencil class="h-3.5 w-3.5" />
          </Button>
          <Button size="sm" variant="secondary" onclick={() => acceptAll(c)}>
            <Check class="mr-1 h-3.5 w-3.5" /> Accept all
          </Button>
          <Button size="sm" variant="destructive" onclick={() => rejectAll(c)}>
            <X class="mr-1 h-3.5 w-3.5" /> Reject all
          </Button>
        </div>
      </div>

      {#if editing === c.id}
        <div class="space-y-2 p-3">
          <textarea
            bind:value={draft}
            rows={12}
            class="w-full rounded border border-border bg-background p-2 font-mono text-xs"
          ></textarea>
          <div class="flex gap-2">
            <Button size="sm" onclick={() => saveEdit(c)}>Save to disk</Button>
            <Button size="sm" variant="ghost" onclick={() => (editing = null)}>Cancel</Button>
          </div>
        </div>
      {:else}
        <div class="divide-y divide-border">
          {#each c.hunks as h (h.index)}
            <div class="p-2">
              <div class="mb-1 flex items-center justify-between">
                <span class="text-[10px] text-muted-foreground">
                  hunk {h.index + 1} · line {h.before_start + 1}
                </span>
                <Button
                  size="sm"
                  variant={h.accepted ? "ghost" : "secondary"}
                  class="h-6 text-[10px]"
                  onclick={() => toggleHunk(c, h)}
                >
                  {h.accepted ? "Reject hunk" : "Restore hunk"}
                </Button>
              </div>
              {#if h.before_lines.length}
                <pre class="overflow-auto rounded bg-red-500/10 px-2 py-1 text-xs {h.accepted
                  ? ''
                  : 'opacity-50'}"><code>{display(h.before_lines).replace(/^/gm, "- ")}</code></pre>
              {/if}
              {#if h.after_lines.length}
                <pre class="overflow-auto rounded bg-green-500/10 px-2 py-1 text-xs {h.accepted
                  ? ''
                  : 'opacity-50 line-through'}"><code
                    >{display(h.after_lines).replace(/^/gm, "+ ")}</code
                  ></pre>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>
