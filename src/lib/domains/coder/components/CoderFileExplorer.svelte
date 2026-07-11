<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ChevronRight, ExternalLink, File, Folder, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { coderWorkspaceStore } from "../state/coderWorkspaceStore.svelte.js";

  interface Props {
    workspaceRoot: string;
  }

  interface DirEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  interface Node extends DirEntry {
    depth: number;
    expanded: boolean;
    loading: boolean;
    error: string | null;
    children: Node[] | null;
  }

  let { workspaceRoot }: Props = $props();

  let root = $state<Node[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function fetchEntries(path: string): Promise<DirEntry[]> {
    return invoke<DirEntry[]>("coder_list_dir", {
      workspaceRoot,
      path: path === "." ? null : path,
    });
  }

  function toNodes(entries: DirEntry[], depth: number): Node[] {
    return entries
      .slice()
      .sort((a, b) => Number(b.is_dir) - Number(a.is_dir) || a.name.localeCompare(b.name))
      .map((e) => ({ ...e, depth, expanded: false, loading: false, error: null, children: null }));
  }

  async function loadRoot() {
    if (!workspaceRoot.trim()) return;
    loading = true;
    error = null;
    try {
      root = toNodes(await fetchEntries("."), 0);
    } catch (e) {
      error = String(e);
      root = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (workspaceRoot) void loadRoot();
  });

  async function toggle(node: Node) {
    if (!node.is_dir) {
      coderWorkspaceStore.openFile(node.path, node.name);
      return;
    }
    node.expanded = !node.expanded;
    if (node.expanded && node.children === null) {
      node.loading = true;
      node.error = null;
      try {
        node.children = toNodes(await fetchEntries(node.path), node.depth + 1);
      } catch (e) {
        node.error = String(e);
        node.children = [];
      } finally {
        node.loading = false;
      }
    }
  }

  async function openInExplorer(path: string, e: MouseEvent) {
    e.stopPropagation();
    try {
      await invoke("coder_open_in_explorer", { workspaceRoot, path });
    } catch (e) {
      error = String(e);
    }
  }

  function flatten(nodes: Node[]): Node[] {
    const out: Node[] = [];
    for (const n of nodes) {
      out.push(n);
      if (n.is_dir && n.expanded && n.children) out.push(...flatten(n.children));
    }
    return out;
  }

  const visible = $derived(flatten(root));
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="divider-edge-b divider-edge-full flex items-center gap-1 px-3 py-2 text-xs">
    <span class="truncate font-mono text-muted-foreground">
      {workspaceRoot.split(/[/\\]/).pop()}
    </span>
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      class="ml-auto h-6 w-6"
      title="Refresh"
      aria-label="Refresh"
      onclick={loadRoot}
    >
      <RefreshCw class="h-3.5 w-3.5" />
    </Button>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto p-2">
    {#if loading}
      <p class="text-xs text-muted-foreground">Loading…</p>
    {:else if error}
      <p class="text-xs text-destructive">{error}</p>
    {:else if visible.length === 0}
      <p class="text-xs text-muted-foreground">Empty directory</p>
    {:else}
      <ul class="space-y-0.5">
        {#each visible as node (node.path)}
          <li>
            <div class="group flex items-center gap-1">
              <Button
                type="button"
                variant="ghost"
                class="h-auto min-w-0 flex-1 justify-start gap-1.5 rounded px-2 py-1 text-xs hover:bg-muted/60"
                style={`padding-left: ${node.depth * 14 + 8}px`}
                onclick={() => toggle(node)}
              >
                {#if node.is_dir}
                  <ChevronRight
                    class={`h-3 w-3 shrink-0 opacity-40 transition-transform ${node.expanded ? "rotate-90" : ""}`}
                  />
                  <Folder class="h-3.5 w-3.5 shrink-0 text-amber-600/80" />
                {:else}
                  <span class="w-3"></span>
                  <File class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                {/if}
                <span class="truncate">{node.name}</span>
                {#if node.loading}
                  <span class="text-muted-foreground">…</span>
                {/if}
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                class="h-7 w-7 opacity-0 transition-opacity group-hover:opacity-100"
                title="Open in file explorer"
                aria-label="Open in file explorer"
                onclick={(e) => openInExplorer(node.path, e)}
              >
                <ExternalLink class="h-3.5 w-3.5" />
              </Button>
            </div>
            {#if node.is_dir && node.expanded && node.error}
              <p class="pl-8 text-xs text-destructive">{node.error}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>
