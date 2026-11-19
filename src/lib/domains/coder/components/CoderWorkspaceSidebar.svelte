<script lang="ts">
  import GitBranch from '~icons/fa6-solid/code-branch';
  import Globe from '~icons/fa6-solid/globe';
  import TerminalIcon from '~icons/fa6-solid/terminal';
  import FileText from '~icons/fa6-solid/file-lines';
  import Plus from '~icons/fa6-solid/plus';
  import Bot from '~icons/fa6-solid/robot';
  import { Button } from '$lib/components/ui/button';
  import { useCoderWorkspace } from '../state/coderWorkspaceStore.svelte.js';
  import { coderTerminalStore } from '../state/coderTerminalStore.svelte.js';
  import { coderSession } from '../state/coderSession.svelte.js';
  import type { GitDiffStats } from '../types.js';

  interface Props {
    threadId: string | null;
    workspaceRoot: string;
    workspaceName: string;
    gitStats?: GitDiffStats | null;
    pendingChangeCount?: number;
    /** Hide the "Agent changes" entry for hosts with no coder change feed. */
    agentChanges?: boolean;
  }

  let {
    threadId,
    workspaceRoot,
    workspaceName,
    gitStats = null,
    pendingChangeCount = 0,
    agentChanges = true,
  }: Props = $props();
  const coderWorkspaceStore = useCoderWorkspace();

  const fileTabs = $derived(coderWorkspaceStore.openTabs.filter((t) => t.panel === 'file'));

  const terminalTabs = $derived.by(() => {
    if (!threadId) return [];
    coderTerminalStore.revision;
    return coderTerminalStore.tabsFor(threadId);
  });

  function createTerminal() {
    if (!threadId || !workspaceRoot) return;
    const tab = coderTerminalStore.createTab(threadId, {
      workspaceRoot,
      createdBy: 'user',
      kind: 'interactive',
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
    return parts.join(' ') || `${gitStats.changedFiles} files`;
  });
</script>

{#snippet row(
  Icon: typeof Bot,
  label: string,
  badge: string | null,
  badgeClass: string,
  active: boolean,
  onclick: () => void
)}
  <Button
    type="button"
    variant="ghost"
    class="h-7 w-full justify-start gap-1.5 rounded px-2 text-xs {active
      ? 'bg-muted text-foreground'
      : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
    {onclick}
  >
    <Icon class="h-3.5 w-3.5 shrink-0" />
    <span class="flex-1 truncate text-left">{label}</span>
    {#if badge}
      <span class="font-mono text-[11px] {badgeClass}">{badge}</span>
    {/if}
  </Button>
{/snippet}

<aside class="divider-edge-l flex w-48 shrink-0 flex-col overflow-hidden bg-muted/20">
  <div class="min-h-0 flex-1 space-y-0.5 overflow-y-auto p-1.5 text-sm">
    {#if agentChanges}
      {@render row(
        Bot,
        'Agent changes',
        agentDiffLabel,
        'text-amber-600 dark:text-amber-400',
        coderWorkspaceStore.activeTabId === 'changes',
        () => {
          coderWorkspaceStore.openChanges();
          if (threadId) coderSession.refreshChanges(threadId);
        }
      )}
    {/if}
    {@render row(
      GitBranch,
      'Git changes',
      gitDiffLabel,
      'text-diff-add-foreground',
      coderWorkspaceStore.activeTabId === 'git-changes',
      () => coderWorkspaceStore.openGitChanges()
    )}
    {@render row(Globe, 'Browser', null, '', coderWorkspaceStore.activeTabId === 'browser', () =>
      coderWorkspaceStore.openBrowser()
    )}
    {@render row(FileText, 'Files', null, '', coderWorkspaceStore.activeTabId === 'files', () =>
      coderWorkspaceStore.openFiles()
    )}

    {#each fileTabs as tab (tab.id)}
      {@render row(FileText, tab.label, null, '', coderWorkspaceStore.activeTabId === tab.id, () =>
        coderWorkspaceStore.selectTab(tab.id)
      )}
    {/each}

    <!-- Terminals: always listed, no extra click to reach one -->
    <div class="flex items-center gap-0.5 pt-1.5">
      <span
        class="flex-1 px-2 text-[11px] font-medium tracking-wide text-muted-foreground uppercase"
      >
        Terminals
      </span>
      {#if threadId}
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          class="h-5 w-5 shrink-0 text-muted-foreground"
          title="New terminal"
          onclick={createTerminal}
        >
          <Plus class="h-3 w-3" />
        </Button>
      {/if}
    </div>
    {#each terminalTabs as tab (tab.id)}
      <Button
        type="button"
        variant="ghost"
        class="h-7 w-full justify-start gap-1.5 rounded px-2 text-xs {coderWorkspaceStore.activeTerminalId() ===
        tab.id
          ? 'bg-muted text-foreground'
          : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
        onclick={() => openTerminalTab(tab.id, tab.label)}
      >
        {#if tab.running}
          <span class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-primary"></span>
        {:else}
          <TerminalIcon class="h-3 w-3 shrink-0 opacity-50" />
        {/if}
        <span class="truncate">{tab.label}</span>
      </Button>
    {:else}
      <p class="px-2 py-1 text-xs text-muted-foreground">
        {threadId ? 'None open' : 'Start a session first'}
      </p>
    {/each}
  </div>
</aside>
