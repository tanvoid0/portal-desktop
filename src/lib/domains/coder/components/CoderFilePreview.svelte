<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    workspaceRoot: string;
    filePath: string;
  }

  let { workspaceRoot, filePath }: Props = $props();

  let original = $state<string | null>(null);
  let content = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let gutter = $state<HTMLDivElement | null>(null);
  let textarea = $state<HTMLTextAreaElement | null>(null);

  const dirty = $derived(original !== null && content !== original);
  const lineCount = $derived(content.length ? content.split("\n").length : 1);

  async function load() {
    loading = true;
    error = null;
    original = null;
    try {
      const text = await invoke<string>("coder_read_file", { workspaceRoot, path: filePath });
      original = text;
      content = text;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function save() {
    if (!dirty || saving) return;
    saving = true;
    error = null;
    try {
      await invoke("coder_write_file", { workspaceRoot, path: filePath, content });
      original = content;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      void save();
    }
  }

  function syncScroll() {
    if (gutter && textarea) gutter.scrollTop = textarea.scrollTop;
  }

  $effect(() => {
    if (workspaceRoot && filePath) void load();
  });
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="divider-edge-b divider-edge-full flex items-center gap-2 px-3 py-2 text-xs">
    <span class="truncate font-mono text-muted-foreground">
      {filePath}{dirty ? " •" : ""}
    </span>
    <Button
      type="button"
      variant="ghost"
      size="sm"
      class="ml-auto h-6 gap-1 text-xs"
      disabled={!dirty || saving}
      onclick={save}
    >
      {saving ? "Saving…" : "Save"}
    </Button>
  </div>
  <div class="min-h-0 flex-1 overflow-hidden">
    {#if loading}
      <p class="p-3 text-xs text-muted-foreground">Loading…</p>
    {:else if error}
      <p class="p-3 text-xs text-destructive">{error}</p>
    {:else}
      <div class="flex h-full min-h-0">
        <div
          bind:this={gutter}
          class="select-none overflow-hidden bg-muted/30 py-2 text-right font-mono text-xs text-muted-foreground"
        >
          {#each { length: lineCount } as _, i (i)}
            <div class="px-2 leading-relaxed">{i + 1}</div>
          {/each}
        </div>
        <textarea
          bind:this={textarea}
          bind:value={content}
          onscroll={syncScroll}
          onkeydown={onKeydown}
          spellcheck="false"
          class="h-full min-h-0 flex-1 resize-none overflow-auto bg-transparent p-2 font-mono text-xs leading-relaxed outline-none"
        ></textarea>
      </div>
    {/if}
  </div>
</div>
