<script lang="ts">
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { MessageSquare, MessageCircle } from "@lucide/svelte";
  import ChatMessage from "./ChatMessage.svelte";
  import ChatInput from "./ChatInput.svelte";
  import ChatCatalogSelectors from "./ChatCatalogSelectors.svelte";
  import type {
    ChatMessage as ChatMessageType,
    ProviderType,
  } from "../../types/index.js";
  import { aiChatService } from "../../services/aiChatService.js";

  interface Props {
    messages?: ChatMessageType[];
    onSendMessage?: (message: string) => void | Promise<void>;
    onSendMessageWithHistory?: (
      message: string,
      history: ChatMessageType[],
    ) => void | Promise<void>;
    isLoading?: boolean;
    placeholder?: string;
    title?: string;
    class?: string;
    conversationId?: string;
    showSelectors?: boolean;
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
  }

  let {
    messages = $bindable<ChatMessageType[]>([]),
    onSendMessage,
    onSendMessageWithHistory,
    isLoading = $bindable(false),
    placeholder = "Type your message...",
    title = "Chat",
    class: className = "",
    conversationId,
    showSelectors = true,
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
  }: Props = $props();

  let messageInput = $state("");
  let messagesContainer: HTMLElement | null = $state(null);
  let scrollViewport: HTMLElement | null = $state(null);

  async function handleSend() {
    if (!messageInput.trim() || isLoading) return;

    const currentMessage = messageInput.trim();
    messageInput = "";

    if (onSendMessageWithHistory) {
      // Don't add message here - parent will handle it
      isLoading = true;
      try {
        await onSendMessageWithHistory(currentMessage, messages);
      } finally {
        isLoading = false;
      }
    } else if (onSendMessage) {
      // Don't add message here - parent will handle it
      isLoading = true;
      try {
        await onSendMessage(currentMessage);
      } finally {
        isLoading = false;
      }
    } else {
      // Use default AI chat service - only add message here if no callback provided
      const userMessage: ChatMessageType = {
        role: "user",
        content: currentMessage,
        timestamp: new Date(),
      };
      messages = [...messages, userMessage];

      isLoading = true;
      try {
        const response = await aiChatService.sendMessage(
          currentMessage,
          messages,
          {
            provider: selectedProvider || undefined,
            llm_provider: selectedBackendProvider || undefined,
            conversation_id: conversationId,
            model: selectedModel || undefined,
          },
        );
        const assistantMessage: ChatMessageType = {
          role: "assistant",
          content: response,
          timestamp: new Date(),
        };
        messages = [...messages, assistantMessage];
      } catch (error) {
        console.error("Failed to send message:", error);
        // Remove user message on error
        messages = messages.slice(0, -1);
      } finally {
        isLoading = false;
      }
    }
  }

  $effect(() => {
    // Scroll to bottom when messages change
    if (scrollViewport && messages.length > 0) {
      // Use requestAnimationFrame for smoother scrolling
      requestAnimationFrame(() => {
        if (scrollViewport) {
          scrollViewport.scrollTop = scrollViewport.scrollHeight;
        }
      });
    }
  });
</script>

<Card class="flex h-full flex-col {className}">
  {#if showSelectors}
    <CardHeader class="pb-3">
      <div class="flex items-center justify-between">
        <CardTitle class="flex items-center gap-2 text-lg">
          <MessageSquare class="h-5 w-5" />
          {title}
        </CardTitle>
        <div class="flex items-center gap-2">
          <ChatCatalogSelectors
            bind:selectedProvider
            bind:selectedBackendProvider
            bind:selectedModel
          />
        </div>
      </div>
    </CardHeader>
  {/if}
  <CardContent class="flex min-h-0 flex-1 flex-col overflow-hidden p-0">
    <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollViewport}>
      <div class="space-y-4 px-4 py-4" bind:this={messagesContainer}>
        {#if messages.length === 0}
          <div class="py-8 text-center text-muted-foreground">
            <MessageCircle class="mx-auto mb-2 h-12 w-12 opacity-50" />
            <p class="text-sm">Start a conversation</p>
          </div>
        {:else}
          {#each messages as message, index}
            {@const isLastMessage = index === messages.length - 1}
            {@const shouldShowLoader =
              isLoading &&
              isLastMessage &&
              message.role === "assistant" &&
              !message.content}
            {@const isStreamingMessage =
              isLoading &&
              isLastMessage &&
              message.role === "assistant" &&
              !!message.content}
            <ChatMessage
              {message}
              showLoader={shouldShowLoader}
              isStreaming={isStreamingMessage}
            />
          {/each}
        {/if}
      </div>
    </ScrollArea>
    <div class="divider-edge-t divider-edge-full flex-shrink-0 bg-background">
      <ChatInput
        bind:value={messageInput}
        onSend={handleSend}
        {placeholder}
        disabled={isLoading}
      />
    </div>
  </CardContent>
</Card>
