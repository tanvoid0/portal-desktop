<!-- Coder action row (model picker, retry, panel toggles, copy, commit) shown in the AI tab bar. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Bot from '~icons/fa6-solid/robot';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import GitCommitHorizontal from '~icons/fa6-solid/code-commit';
  import RotateCcw from '~icons/fa6-solid/rotate-left';
  import Copy from '~icons/fa6-solid/copy';
  import Check from '~icons/fa6-solid/check';
  import TerminalIcon from '~icons/fa6-solid/terminal';
  import PanelRightOpen from '~icons/fa6-solid/table-columns';
  import { toast } from '$lib/utils/toast';
  import { isAuthError } from '$lib/domains/ai/utils/catalog.js';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
  import { coderSession } from '../state/coderSession.svelte.js';
  import { coderTerminalStore } from '../state/coderTerminalStore.svelte.js';
  import { coderWorkspaceStore } from '../state/coderWorkspaceStore.svelte.js';
  import type { CoderThread, GitDiffStats } from '../types.js';

  interface Props {
    thread: CoderThread | null | undefined;
    activeThreadId: string | null;
    activeTerminalThread: CoderThread | null | undefined;
    running: boolean;
    showRetry: boolean;
    platformBlocked: boolean;
    pendingChangeCount: number;
    gitStats: GitDiffStats | null;
    canCopyConversation: boolean;
    getConversationText: () => string;
    onCommit: () => void;
  }

  let {
    thread,
    activeThreadId,
    activeTerminalThread,
    running,
    showRetry,
    platformBlocked,
    pendingChangeCount,
    gitStats,
    canCopyConversation,
    getConversationText,
    onCommit,
  }: Props = $props();

  let catalogError = $state<string | null>(null);
  let copied = $state(false);

  const tokenRejected = $derived(isAuthError(catalogError));
  const showChanges = $derived(coderWorkspaceStore.activePanel === 'changes');
  const showGitChanges = $derived(coderWorkspaceStore.activePanel === 'git-changes');
  const terminalOpen = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.terminalOpen;
  });

  async function copyConversation() {
    const text = getConversationText();
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      toast.success('Conversation copied to clipboard');
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      toast.error('Failed to copy conversation');
    }
  }
</script>

<ProviderModelSelector
  bind:selectedProvider={coderSession.selectedProvider}
  bind:selectedBackendProvider={coderSession.selectedBackendProvider}
  bind:selectedModel={coderSession.selectedModel}
  onModelChange={(m) => coderSession.handleModelChange(m)}
  onBackendProviderChange={(p) => coderSession.handleBackendProviderChange(p)}
  modelSelectClass="w-[180px]"
  showInlineError={false}
  onStatusChange={(s) => (catalogError = s.error)}
/>
{#if catalogError && !platformBlocked}
  <span class="text-xs font-medium text-destructive dark:text-red-400" title={catalogError}>
    {tokenRejected ? 'Agent Platform token rejected.' : 'Agent Platform unreachable.'}
    <a href={AI_PROVIDER_SETTINGS_PATH} class="underline">Settings &gt; AI</a>
  </span>
{/if}
{#if showRetry && thread && !running}
  <Button
    size="sm"
    variant="outline"
    class="h-8 gap-1"
    title={platformBlocked ? 'Unavailable: Agent Platform is down' : 'Retry the last message'}
    disabled={platformBlocked}
    onclick={() => coderSession.retry()}
  >
    <RotateCcw class="h-3.5 w-3.5" />
    Retry
  </Button>
{/if}
<Button
  size="sm"
  variant={coderWorkspaceStore.sidebarOpen ? 'secondary' : 'ghost'}
  class="h-8 gap-1"
  title="Workspace sidebar"
  onclick={() => {
    coderWorkspaceStore.sidebarOpen = !coderWorkspaceStore.sidebarOpen;
  }}
>
  <PanelRightOpen class="h-3.5 w-3.5" />
</Button>
<Button
  size="sm"
  variant={terminalOpen ? 'secondary' : 'ghost'}
  class="h-8 gap-1"
  title="Session terminal"
  disabled={!activeThreadId}
  onclick={() => {
    if (!activeThreadId || !activeTerminalThread) return;
    const tab = coderTerminalStore.ensureDefault(
      activeThreadId,
      activeTerminalThread.workspace_root
    );
    coderWorkspaceStore.openTerminal(activeThreadId, tab.id, tab.label);
    coderSession.terminalOpen = true;
  }}
>
  <TerminalIcon class="h-3.5 w-3.5" />
  <span class="hidden xl:inline">Terminal</span>
</Button>
<Button
  size="icon"
  variant="ghost"
  class="h-8 w-8"
  title="Copy conversation for debugging"
  disabled={!canCopyConversation}
  onclick={copyConversation}
>
  {#if copied}
    <Check class="h-3.5 w-3.5 text-green-500" />
  {:else}
    <Copy class="h-3.5 w-3.5" />
  {/if}
</Button>
<Button
  size="sm"
  variant={showChanges ? 'secondary' : 'ghost'}
  class="h-8 gap-1"
  title="Review agent file changes"
  onclick={() => {
    coderWorkspaceStore.openChanges();
    coderSession.refreshChanges(thread?.id);
  }}
>
  <Bot class="h-3.5 w-3.5" />
  {#if pendingChangeCount > 0}
    <span class="rounded-full bg-amber-500 px-1.5 text-[10px] text-white">
      {pendingChangeCount}
    </span>
  {/if}
</Button>
<Button
  size="sm"
  variant={showGitChanges ? 'secondary' : 'ghost'}
  class="h-8 gap-1"
  title="View git working tree changes"
  onclick={() => coderWorkspaceStore.openGitChanges()}
>
  <GitBranch class="h-3.5 w-3.5" />
  {#if gitStats?.hasChanges}
    <span class="font-mono text-[10px] text-green-600 dark:text-green-400">
      {#if gitStats.additions > 0}+{gitStats.additions}{/if}
      {#if gitStats.deletions > 0}-{gitStats.deletions}{/if}
    </span>
  {/if}
</Button>
<Button
  size="sm"
  variant="outline"
  class="h-8 gap-1"
  title="Review and commit git changes"
  disabled={!gitStats?.hasChanges}
  onclick={onCommit}
>
  <GitCommitHorizontal class="h-3.5 w-3.5" />
  <span class="hidden xl:inline">Commit</span>
</Button>
