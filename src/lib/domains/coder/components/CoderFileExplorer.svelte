<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ChevronRight, ExternalLink, File, Folder } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    workspaceRoot: string;
  }

  interface DirEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  let { workspaceRoot }: Props = $props();

  let currentPath = $state(".");
  let entries = $state<DirEntry[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function openInExplorer(path: string) {
    try {
      await invoke("coder_open_in_explorer", {
        workspaceRoot,
        path,
      });
    } catch (e) {
      error = String(e);
    }
  }

  async function loadDir(path: string) {
    if (!workspaceRoot.trim()) return;
    loading = true;
    error = null;
    try {
      entries = await invoke<DirEntry[]>("coder_list_dir", {
        workspaceRoot,
        path: path === "." ? null : path,
      });
      currentPath = path;
    } catch (e) {
      error = String(e);
      entries = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (workspaceRoot) void loadDir(".");
  });

  function parentPath(): string {
    if (currentPath === ".") return ".";
    const parts = currentPath.replace(/\\/g, "/").split("/").filter(Boolean);
    parts.pop();
    return parts.length ? parts.join("/") : ".";
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="divider-edge-b divider-edge-full flex items-center gap-1 px-3 py-2 text-xs">
    {#if currentPath !== "."}
      <Button
        type="button"
        variant="link"
        class="h-auto p-0 text-muted-foreground"
        onclick={() => loadDir(parentPath())}
      >
        ..
      </Button>
      <span class="text-muted-foreground">/</span>
    {/if}
    <span class="truncate font-mono text-muted-foreground">
      {currentPath === "." ? workspaceRoot.split(/[/\\]/).pop() : currentPath}
    </span>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    {#if loading}
      <p class="text-xs text-muted-foreground">Loading…</p>
    {:else if error}
      <p class="text-xs text-destructive">{error}</p>
    {:else if entries.length === 0}
      <p class="text-xs text-muted-foreground">Empty directory</p>
    {:else}
      <ul class="space-y-0.5">
        {#each entries as entry (entry.path)}
          <li>
            {#if entry.is_dir}
              <div class="group flex items-center gap-1">
                <Button
                  type="button"
                  variant="ghost"
                  class="h-auto min-w-0 flex-1 justify-start gap-1.5 rounded px-2 py-1 text-xs hover:bg-muted/60"
                  onclick={() => loadDir(entry.path)}
                >
                  <ChevronRight class="h-3 w-3 shrink-0 opacity-40" />
                  <Folder class="h-3.5 w-3.5 shrink-0 text-amber-600/80" />
                  <span class="truncate">{entry.name}</span>
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  class="h-7 w-7 opacity-0 transition-opacity group-hover:opacity-100"
                  title="Open in file explorer"
                  aria-label="Open in file explorer"
                  onclick={(e) => {
                    e.stopPropagation();
                    void openInExplorer(entry.path);
                  }}
                >
                  <ExternalLink class="h-3.5 w-3.5" />
                </Button>
              </div>
            {:else}
              <div class="group flex items-center gap-1">
                <div
                  class="flex min-w-0 flex-1 items-center gap-1.5 rounded px-2 py-1 pl-7 text-xs text-muted-foreground"
                >
                  <File class="h-3.5 w-3.5 shrink-0" />
                  <span class="truncate">{entry.name}</span>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  class="h-7 w-7 opacity-0 transition-opacity group-hover:opacity-100"
                  title="Open in file explorer"
                  aria-label="Open in file explorer"
                  onclick={(e) => {
                    e.stopPropagation();
                    void openInExplorer(entry.path);
                  }}
                >
                  <ExternalLink class="h-3.5 w-3.5" />
                </Button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>
