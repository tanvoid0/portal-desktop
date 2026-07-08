<script lang="ts">
  import { X, Maximize2, MessageSquare } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { coderWorkspaceStore } from "../state/coderWorkspaceStore.svelte.js";
  import CoderTerminalPanel from "./CoderTerminalPanel.svelte";
  import CoderFileExplorer from "./CoderFileExplorer.svelte";
  import CoderBrowserPanel from "./CoderBrowserPanel.svelte";
  import ChangesPanel from "./ChangesPanel.svelte";
  import GitChangesPanel from "./GitChangesPanel.svelte";
  import type { FileChange } from "../types.js";

  interface Props {
    threadId: string | null;
    workspaceRoot: string;
    changes: FileChange[];
    onRefreshChanges: () => void;
    onCommit?: () => void;
  }

  let { threadId, workspaceRoot, changes, onRefreshChanges, onCommit }: Props = $props();

  const panel = $derived(coderWorkspaceStore.activePanel);
  const activeTab = $derived(
    coderWorkspaceStore.openTabs.find(
      (t) => t.id === coderWorkspaceStore.activeTabId,
    ),
  );

  const title = $derived(activeTab?.label ?? "Workspace");

  function backToChat() {
    coderWorkspaceStore.openChat();
  }
</script>

<div class="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden border-l border-border bg-background">
  <div
    class="flex shrink-0 items-center justify-between gap-2 border-b border-border px-3 py-2"
  >
    <div class="flex min-w-0 items-center gap-2">
      <h3 class="truncate text-sm font-medium">{title}</h3>
    </div>
    <div class="flex items-center gap-1">
      <Button
        size="sm"
        variant="ghost"
        class="h-7 gap-1 text-xs"
        title="Back to chat"
        onclick={backToChat}
      >
        <MessageSquare class="h-3.5 w-3.5" />
        Chat
      </Button>
      {#if activeTab}
        <Button
          size="icon"
          variant="ghost"
          class="h-7 w-7"
          title="Close tab"
          onclick={() => coderWorkspaceStore.closeTab(activeTab.id)}
        >
          <X class="h-3.5 w-3.5" />
        </Button>
      {/if}
    </div>
  </div>

  <div class="min-h-0 flex-1">
    {#if panel === "terminal" && threadId && workspaceRoot}
      <CoderTerminalPanel
        {threadId}
        {workspaceRoot}
        activeTerminalId={coderWorkspaceStore.activeTerminalId()}
      />
    {:else if panel === "files" && workspaceRoot}
      <CoderFileExplorer {workspaceRoot} />
    {:else if panel === "browser"}
      <CoderBrowserPanel />
    {:else if panel === "changes"}
      <div class="h-full overflow-auto p-4">
        <ChangesPanel {changes} onRefresh={onRefreshChanges} />
      </div>
    {:else if panel === "git-changes" && workspaceRoot}
      <div class="h-full overflow-auto p-4">
        <GitChangesPanel {workspaceRoot} {onCommit} />
      </div>
    {/if}
  </div>
</div>
