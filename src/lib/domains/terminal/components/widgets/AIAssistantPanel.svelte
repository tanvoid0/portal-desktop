<script lang="ts">
  import { onMount } from "svelte";
  import { Bot, Send, Sparkles } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { aiChatService } from "$lib/domains/ai/services/aiChatService";
  import { commandBlockStore } from "../../stores/commandBlockStore";

  interface Props {
    tabId: string;
  }

  let { tabId }: Props = $props();

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

    try {
      await aiChatService.streamMessage(question, [], {
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
        onError: (error) => {
          const errorMessage = `Error: ${error.message}`;
          commandBlockStore.appendOutput(tabId, blockId, `\n\n${errorMessage}`);
          commandBlockStore.completeBlock(tabId, blockId, 1);
          messages = [
            ...messages,
            {
              role: "assistant",
              content: errorMessage,
              timestamp: new Date(),
            },
          ];
          isLoading = false;
        },
      });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Unknown error";
      messages = [
        ...messages,
        {
          role: "assistant",
          content: message,
          timestamp: new Date(),
        },
      ];
      isLoading = false;
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="flex items-center gap-2 border-b border-gray-700 p-2">
    <Bot class="h-4 w-4 text-purple-400" />
    <div class="text-sm font-medium text-gray-200">AI Assistant</div>
  </div>

  <div class="min-h-0 flex-1 space-y-2 overflow-y-auto p-3">
    {#if messages.length === 0}
      <p class="text-xs text-gray-400">
        Ask questions about commands, get suggestions, or use <code>/ai</code> in the input bar.
      </p>
    {:else}
      {#each messages as msg (msg.timestamp.getTime() + msg.role)}
        <div
          class="rounded-lg p-2 text-sm {msg.role === 'user'
            ? 'bg-gray-700 text-gray-100'
            : 'bg-purple-950/40 text-gray-200'}"
        >
          {msg.content}
        </div>
      {/each}
    {/if}
  </div>

  <div class="border-t border-gray-700 p-2">
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
        class="border-gray-600 bg-gray-900 text-gray-100"
      />
      <Button type="submit" size="sm" disabled={isLoading || !inputValue.trim()}>
        <Send class="h-4 w-4" />
      </Button>
    </form>
  </div>
</div>
