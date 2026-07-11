<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Plus, X, Terminal as TerminalIcon } from "@lucide/svelte";
  import { coderTerminalStore } from "../state/coderTerminalStore.svelte.js";
  import { coderWorkspaceStore } from "../state/coderWorkspaceStore.svelte.js";
  import CoderSessionTerminal from "./CoderSessionTerminal.svelte";

  interface Props {
    threadId: string;
    workspaceRoot: string;
    open?: boolean;
    /** When set, selects this terminal tab. */
    activeTerminalId?: string | null;
  }

  let {
    threadId,
    workspaceRoot,
    open = true,
    activeTerminalId = null,
  }: Props = $props();

  const tabs = $derived.by(() => {
    coderTerminalStore.revision;
    return coderTerminalStore.tabsFor(threadId);
  });

  const activeId = $derived.by(() => {
    coderTerminalStore.revision;
    return coderTerminalStore.activeId(threadId);
  });

  function createTerminal() {
    const tab = coderTerminalStore.createTab(threadId, {
      workspaceRoot,
      createdBy: "user",
      kind: "interactive",
    });
    coderWorkspaceStore.openTerminal(threadId, tab.id, tab.label);
  }

  function selectTab(id: string) {
    coderTerminalStore.setActive(threadId, id);
  }

  function closeTab(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (tabs.length <= 1) return;
    coderTerminalStore.removeTab(threadId, id);
  }

  $effect(() => {
    if (
      activeTerminalId &&
      coderTerminalStore.activeByThread[threadId] !== activeTerminalId
    ) {
      coderTerminalStore.setActive(threadId, activeTerminalId);
    }
  });

  $effect(() => {
    if (open && threadId && workspaceRoot && tabs.length === 0) {
      coderTerminalStore.ensureDefault(threadId, workspaceRoot);
    }
  });
</script>

{#if open}
  <div class="flex h-full min-h-0 flex-col">
    <div
      class="divider-edge-b divider-edge-full flex shrink-0 items-center gap-1 overflow-x-auto px-2 py-1"
    >
      {#each tabs as tab (tab.id)}
        <Button
          type="button"
          variant="ghost"
          class="h-auto max-w-[140px] gap-1 rounded px-2 py-1 text-xs {activeId ===
          tab.id
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:bg-muted/60'}"
          onclick={() => selectTab(tab.id)}
          title={tab.label}
        >
          {#if tab.running}
            <span
              class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-primary"
            ></span>
          {:else}
            <TerminalIcon class="h-3 w-3 shrink-0 opacity-60" />
          {/if}
          <span class="truncate">{tab.label}</span>
          {#if tabs.length > 1}
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              class="ml-0.5 hidden h-5 w-5 group-hover:inline-flex"
              onclick={(e) => closeTab(tab.id, e)}
              title="Close tab"
            >
              <X class="h-3 w-3" />
            </Button>
          {/if}
        </Button>
      {/each}
      <Button
        size="icon"
        variant="ghost"
        class="h-7 w-7 shrink-0"
        title="New terminal"
        onclick={createTerminal}
      >
        <Plus class="h-3.5 w-3.5" />
      </Button>
    </div>

    <div class="relative min-h-0 flex-1">
      {#each tabs as tab (tab.id)}
        <div
          class="absolute inset-0"
          class:hidden={tab.id !== activeId}
          aria-hidden={tab.id !== activeId}
        >
          <CoderSessionTerminal
            {threadId}
            {tab}
            {workspaceRoot}
            visible={tab.id === activeId && open}
          />
        </div>
      {/each}
    </div>
  </div>
{/if}
