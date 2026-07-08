<script lang="ts">
  import {
    ChevronRight,
    GitBranch,
    Globe,
    Terminal as TerminalIcon,
    FileText,
    Plus,
    Bot,
  } from "@lucide/svelte";
  import { coderWorkspaceStore } from "../state/coderWorkspaceStore.svelte.js";
  import { coderTerminalStore } from "../state/coderTerminalStore.svelte.js";
  import { coderSession } from "../state/coderSession.svelte.js";
  import type { GitDiffStats } from "../types.js";

  interface Props {
    threadId: string | null;
    workspaceRoot: string;
    workspaceName: string;
    gitStats?: GitDiffStats | null;
    pendingChangeCount?: number;
  }

  let {
    threadId,
    workspaceRoot,
    workspaceName,
    gitStats = null,
    pendingChangeCount = 0,
  }: Props = $props();

  const openTabs = $derived.by(() => {
    coderWorkspaceStore.openTabs;
    return coderWorkspaceStore.openTabs;
  });

  const terminalTabs = $derived.by(() => {
    if (!threadId) return [];
    coderTerminalStore.revision;
    return coderTerminalStore.tabsFor(threadId);
  });

  const terminalExpanded = $derived(coderWorkspaceStore.isExpanded("terminal"));
  const filesExpanded = $derived(coderWorkspaceStore.isExpanded("files"));
  const browserExpanded = $derived(coderWorkspaceStore.isExpanded("browser"));
  const changesExpanded = $derived(coderWorkspaceStore.isExpanded("changes"));
  const gitChangesExpanded = $derived(coderWorkspaceStore.isExpanded("git-changes"));

  function createTerminal() {
    if (!threadId || !workspaceRoot) return;
    const tab = coderTerminalStore.createTab(threadId, {
      workspaceRoot,
      createdBy: "user",
      kind: "interactive",
    });
    coderWorkspaceStore.openTerminal(threadId, tab.id, tab.label);
  }

  function openTerminalTab(terminalId: string, label: string) {
    if (!threadId) return;
    coderTerminalStore.setActive(threadId, terminalId);
    coderWorkspaceStore.openTerminal(threadId, terminalId, label);
  }

  const agentDiffLabel = $derived.by(() => {
    if (pendingChangeCount === 0) return null;
    return `${pendingChangeCount} pending`;
  });

  const gitDiffLabel = $derived.by(() => {
    if (!gitStats?.isRepo) return null;
    if (!gitStats.hasChanges) return null;
    const parts: string[] = [];
    if (gitStats.additions > 0) parts.push(`+${gitStats.additions}`);
    if (gitStats.deletions > 0) parts.push(`-${gitStats.deletions}`);
    return parts.join(" ") || `${gitStats.changedFiles} files`;
  });
</script>

<aside
  class="flex w-56 shrink-0 flex-col overflow-hidden border-l border-border bg-muted/20"
>
  <div class="flex items-center border-b border-border px-2 py-1.5">
    <span class="text-[11px] font-medium text-muted-foreground">Workspace</span>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto text-sm">
    {#if openTabs.length > 0}
      <div class="border-b border-border px-2 py-2">
        <div class="mb-1 px-1 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
          Open tabs
        </div>
        <ul class="space-y-0.5">
          {#each openTabs as tab (tab.id)}
            <li>
              <button
                type="button"
                class="flex w-full items-center gap-1.5 rounded px-2 py-1 text-left text-xs transition-colors {coderWorkspaceStore.activeTabId ===
                tab.id
                  ? 'bg-muted text-foreground'
                  : 'text-muted-foreground hover:bg-muted/60'}"
                onclick={() => coderWorkspaceStore.selectTab(tab.id)}
              >
                {#if tab.panel === "terminal"}
                  <TerminalIcon class="h-3.5 w-3.5 shrink-0" />
                {:else if tab.panel === "changes"}
                  <Bot class="h-3.5 w-3.5 shrink-0" />
                {:else if tab.panel === "git-changes"}
                  <GitBranch class="h-3.5 w-3.5 shrink-0" />
                {:else if tab.panel === "browser"}
                  <Globe class="h-3.5 w-3.5 shrink-0" />
                {:else}
                  <FileText class="h-3.5 w-3.5 shrink-0" />
                {/if}
                <span class="truncate">{tab.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <div class="px-2 py-2">
      <div class="mb-1 px-1 text-[10px] font-medium text-muted-foreground">
        On {workspaceName || "workspace"}
      </div>

      <!-- Agent changes -->
      <div class="rounded-md">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-left text-xs hover:bg-muted/60"
          onclick={() => {
            coderWorkspaceStore.expandSection("changes");
            coderWorkspaceStore.openChanges();
            if (threadId) coderSession.refreshChanges(threadId);
          }}
        >
          <ChevronRight
            class="h-3.5 w-3.5 shrink-0 transition-transform {changesExpanded
              ? 'rotate-90'
              : ''}"
          />
          <Bot class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span class="flex-1">Agent changes</span>
          {#if agentDiffLabel}
            <span class="font-mono text-[10px] text-amber-600 dark:text-amber-400">
              {agentDiffLabel}
            </span>
          {/if}
        </button>
      </div>

      <!-- Git changes -->
      <div class="rounded-md">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-left text-xs hover:bg-muted/60"
          onclick={() => {
            coderWorkspaceStore.expandSection("git-changes");
            coderWorkspaceStore.openGitChanges();
          }}
        >
          <ChevronRight
            class="h-3.5 w-3.5 shrink-0 transition-transform {gitChangesExpanded
              ? 'rotate-90'
              : ''}"
          />
          <GitBranch class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span class="flex-1">Git changes</span>
          {#if gitDiffLabel}
            <span class="font-mono text-[10px] text-green-600 dark:text-green-400">
              {gitDiffLabel}
            </span>
          {/if}
        </button>
      </div>

      <!-- Browser -->
      <div class="rounded-md">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-left text-xs hover:bg-muted/60"
          onclick={() => {
            coderWorkspaceStore.expandSection("browser");
            coderWorkspaceStore.openBrowser();
          }}
        >
          <ChevronRight
            class="h-3.5 w-3.5 shrink-0 transition-transform {browserExpanded
              ? 'rotate-90'
              : ''}"
          />
          <Globe class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span>Browser</span>
        </button>
      </div>

      <!-- Terminal -->
      <div class="rounded-md">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-left text-xs hover:bg-muted/60"
          onclick={() => {
            coderWorkspaceStore.toggleSection("terminal");
            if (terminalTabs.length > 0) {
              const tab = terminalTabs.find(
                (t) => t.id === coderWorkspaceStore.activeTerminalId(),
              ) ?? terminalTabs[0];
              openTerminalTab(tab.id, tab.label);
            } else if (threadId) {
              createTerminal();
            }
          }}
        >
          <ChevronRight
            class="h-3.5 w-3.5 shrink-0 transition-transform {terminalExpanded
              ? 'rotate-90'
              : ''}"
          />
          <TerminalIcon class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span class="flex-1">
            {terminalTabs.length}
            {terminalTabs.length === 1 ? "Terminal" : "Terminals"}
          </span>
          {#if threadId}
            <span
              role="button"
              tabindex="0"
              class="rounded p-0.5 hover:bg-background"
              title="New terminal"
              onclick={(e) => {
                e.stopPropagation();
                createTerminal();
              }}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  e.stopPropagation();
                  createTerminal();
                }
              }}
            >
              <Plus class="h-3.5 w-3.5" />
            </span>
          {/if}
        </button>
        {#if terminalExpanded}
          <ul class="mb-1 ml-6 space-y-0.5 border-l border-border pl-2">
            {#each terminalTabs as tab (tab.id)}
              <li>
                <button
                  type="button"
                  class="flex w-full items-center gap-1.5 rounded px-2 py-1 text-left text-xs transition-colors {coderWorkspaceStore.activeTerminalId() ===
                  tab.id
                    ? 'bg-muted text-foreground'
                    : 'text-muted-foreground hover:bg-muted/60'}"
                  onclick={() => openTerminalTab(tab.id, tab.label)}
                >
                  {#if tab.running}
                    <span
                      class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-primary"
                    ></span>
                  {:else}
                    <TerminalIcon class="h-3 w-3 shrink-0 opacity-50" />
                  {/if}
                  <span class="truncate">{tab.label}</span>
                </button>
              </li>
            {:else}
              <li class="px-2 py-1 text-[11px] text-muted-foreground">
                {#if threadId}
                  <button
                    type="button"
                    class="text-primary hover:underline"
                    onclick={createTerminal}
                  >
                    Create terminal
                  </button>
                {:else}
                  Start a session first
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <!-- Files -->
      <div class="rounded-md">
        <button
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-2 py-1.5 text-left text-xs hover:bg-muted/60"
          onclick={() => {
            coderWorkspaceStore.expandSection("files");
            coderWorkspaceStore.openFiles();
          }}
        >
          <ChevronRight
            class="h-3.5 w-3.5 shrink-0 transition-transform {filesExpanded
              ? 'rotate-90'
              : ''}"
          />
          <FileText class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span>Files</span>
        </button>
      </div>
    </div>
  </div>
</aside>
