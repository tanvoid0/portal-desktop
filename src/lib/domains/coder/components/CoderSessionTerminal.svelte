<script lang="ts">
  import {
    Terminal,
    defaultTerminalConfig,
    type TerminalConfig,
  } from "$lib/domains/terminal";
  import type { CoderTerminalTab } from "../state/coderTerminalStore.svelte.js";
  import {
    registerCoderTerminal,
  } from "../services/coderTerminalCoordinator.js";

  interface Props {
    threadId: string;
    tab: CoderTerminalTab;
    workspaceRoot: string;
    visible?: boolean;
  }

  let { threadId, tab, workspaceRoot, visible = true }: Props = $props();

  let terminal = $state<ReturnType<typeof Terminal> | null>(null);

  const settings = $derived<TerminalConfig>({
    ...defaultTerminalConfig,
    workingDirectory: workspaceRoot || defaultTerminalConfig.workingDirectory,
    scrollbackLines: 5000,
    fontSize: 12,
  });

  $effect(() => {
    registerCoderTerminal(threadId, tab.id, terminal);
    return () => registerCoderTerminal(threadId, tab.id, null);
  });

  $effect(() => {
    if (visible && terminal) {
      requestAnimationFrame(() => terminal?.fit());
    }
  });
</script>

<div class="coder-session-terminal h-full min-h-0 w-full">
  <Terminal
    bind:this={terminal}
    tabId={tab.tabId}
    {settings}
    mode="interactive"
    killOnDestroy={false}
    class="h-full"
  />
</div>
