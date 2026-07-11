<script lang="ts">
  import { Bot, Send } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { aiChatService } from "$lib/domains/ai/services/aiChatService";
  import { commandBlockStore } from "../../stores/commandBlockStore";
  import { buildTerminalContext } from "../../services/terminalAiContext";
  import AiResponse from "../ai/AiResponse.svelte";

  interface Props {
    tabId: string;
    /** Shell of the active session, for context + suggestion syntax. */
    shell?: string;
    /** Starting/fallback working directory for this tab. */
    workingDirectory?: string;
    /** Run an AI-suggested command in the active terminal session. */
    onRunCommand?: (command: string) => void;
  }

  let { tabId, shell, workingDirectory, onRunCommand }: Props = $props();

  let messages = $state<
    Array<{ role: "user" | "assistant"; content: string; timestamp: Date }>
  >([]);
  let inputValue = $state("");
  let isLoading = $state(false);

  async function handleSend() {
    const question = inputValue.trim();
    if (!question || isLoading) return;

    messages = [
      ...messages,
      { role: "user", content: question, timestamp: new Date() },
    ];
    inputValue = "";
    isLoading = true;

    let assistantContent = "";
    const blockId = commandBlockStore.addBlock(tabId, {
      command: `/ai ${question}`,
      output: "",
      source: "ai",
      status: "running",
    });

    const context = buildTerminalContext(tabId, { shell, workingDirectory });
    const message = `${context}\n\n---\nUser request: ${question}`;

    // streamMessage both invokes onError AND rejects — surface the error once.
    let failed = false;
    const fail = (msg: string) => {
      if (failed) return;
      failed = true;
      const errorMessage = `Error: ${msg}`;
      commandBlockStore.appendOutput(tabId, blockId, `\n\n${errorMessage}`);
      commandBlockStore.completeBlock(tabId, blockId, 1);
      messages = [
        ...messages,
        { role: "assistant", content: errorMessage, timestamp: new Date() },
      ];
      isLoading = false;
    };

    try {
      await aiChatService.streamMessage(message, [], {
        onChunk: (chunk) => {
          assistantContent += chunk;
          commandBlockStore.appendOutput(tabId, blockId, chunk);
        },
        onComplete: () => {
          commandBlockStore.completeBlock(tabId, blockId, 0);
          messages = [
            ...messages,
            {
              role: "assistant",
              content: assistantContent,
              timestamp: new Date(),
            },
          ];
          isLoading = false;
        },
        onError: (error) => fail(error.message),
      });
    } catch (error) {
      fail(error instanceof Error ? error.message : "Unknown error");
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="divider-edge-b divider-edge-full flex items-center gap-2 p-2">
    <Bot class="h-4 w-4 text-primary" />
    <div class="text-sm font-medium text-foreground">AI Assistant</div>
  </div>

  <div class="min-h-0 flex-1 space-y-2 overflow-y-auto p-3">
    {#if messages.length === 0}
      <p class="text-xs text-muted-foreground">
        Ask about your recent commands, errors, or describe what you want to do
        — the AI sees this tab's command history and suggests runnable commands.
      </p>
    {:else}
      {#each messages as msg (msg.timestamp.getTime() + msg.role)}
        <div
          class="rounded-lg p-2 text-sm {msg.role === 'user'
            ? 'bg-primary text-primary-foreground'
            : 'border border-border bg-muted/60 text-foreground'}"
        >
          {#if msg.role === "assistant"}
            <AiResponse content={msg.content} {onRunCommand} />
          {:else}
            {msg.content}
          {/if}
        </div>
      {/each}
      {#if isLoading}
        <p class="animate-pulse text-xs text-primary">Thinking…</p>
      {/if}
    {/if}
  </div>

  <div class="divider-edge-t divider-edge-full p-2">
    <form
      class="flex gap-2"
      onsubmit={(e) => {
        e.preventDefault();
        handleSend();
      }}
    >
      <Input
        bind:value={inputValue}
        placeholder="Ask AI..."
        disabled={isLoading}
        class="bg-background"
      />
      <Button type="submit" size="sm" disabled={isLoading || !inputValue.trim()}>
        <Send class="h-4 w-4" />
      </Button>
    </form>
  </div>
</div>
