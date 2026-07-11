<script lang="ts">
  import {
    Terminal,
    defaultTerminalConfig,
    type TerminalConfig,
  } from "$lib/domains/terminal";
  import { coderTerminalTabId } from "../state/coderTerminalStore.svelte.js";

  interface Props {
    threadId: string;
    callId: string;
    command: string;
    output: string;
    failed?: boolean;
    workspaceRoot?: string;
  }

  let {
    threadId,
    callId,
    command,
    output,
    failed = false,
    workspaceRoot = "",
  }: Props = $props();

  let terminal = $state<ReturnType<typeof Terminal> | null>(null);

  const tabId = $derived(coderTerminalTabId(threadId, `cmd-${callId}`));

  const displayContent = $derived.by(() => {
    const prompt = failed ? "\x1b[31m" : "\x1b[90m";
    const body = failed ? "\x1b[31m" : "";
    const reset = "\x1b[0m";
    return `${prompt}$ ${command}${reset}\n\n${body}${output}${reset}\n`;
  });

  const settings = $derived<TerminalConfig>({
    ...defaultTerminalConfig,
    workingDirectory: workspaceRoot || defaultTerminalConfig.workingDirectory,
    scrollbackLines: 2000,
    fontSize: 12,
  });

  $effect(() => {
    if (terminal) requestAnimationFrame(() => terminal?.fit());
  });
</script>

<div class="coder-command-output h-48 min-h-0 w-full overflow-hidden rounded border border-border bg-[#0c0c0c]">
  <Terminal
    bind:this={terminal}
    {tabId}
    {settings}
    mode="display"
    {displayContent}
    killOnDestroy={false}
    class="h-full"
  />
</div>
