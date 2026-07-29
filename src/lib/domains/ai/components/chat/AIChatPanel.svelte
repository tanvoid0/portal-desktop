<script lang="ts">
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Button } from '$lib/components/ui/button';
  import {
    MessageSquare,
    MessageCircle,
    Copy,
    Check,
    RefreshCw,
    Pencil,
    Trash2,
    GitBranch,
  } from '@lucide/svelte';
  import ChatMessage from './ChatMessage.svelte';
  import ChatInput from './ChatInput.svelte';
  import ChatCatalogSelectors from './ChatCatalogSelectors.svelte';
  import type {
    ChatMessage as ChatMessageType,
    ContextUsage,
    ProviderType,
  } from '../../types/index.js';
  import { aiChatService } from '../../services/aiChatService.js';

  interface Props {
    messages?: ChatMessageType[];
    onSendMessage?: (message: string) => void | Promise<void>;
    onSendMessageWithHistory?: (
      message: string,
      history: ChatMessageType[]
    ) => void | Promise<void>;
    /** When set, a stop button replaces send while a response is streaming. */
    onStop?: () => void;
    /** Re-run the last user turn. Enables the regenerate action. */
    onRegenerate?: () => void | Promise<void>;
    /** Drop `messages[index]` and everything after it. Enables edit/delete. */
    onTruncate?: (index: number) => void | Promise<void>;
    /** Fork the thread at `index` into a new conversation. */
    onBranch?: (index: number) => void | Promise<void>;
    /** Block sending and re-running — e.g. the platform is unreachable. */
    sendDisabled?: boolean;
    /** Replaces the composer hint while `sendDisabled`. */
    sendDisabledHint?: string;
    /** Shown above assistant turns on hover — usually the model id. */
    modelLabel?: string | null;
    isLoading?: boolean;
    placeholder?: string;
    title?: string;
    class?: string;
    conversationId?: string;
    showSelectors?: boolean;
    selectedProvider?: ProviderType | null;
    selectedBackendProvider?: string | null;
    selectedModel?: string | null;
    /** Drives the composer's `used / window` token counter. */
    contextUsage?: ContextUsage | null;
    /** Wires the composer wrench to the model-parameters panel. */
    onOpenSettings?: () => void;
    settingsOpen?: boolean;
  }

  let {
    messages = $bindable<ChatMessageType[]>([]),
    onSendMessage,
    onSendMessageWithHistory,
    onStop,
    onRegenerate,
    onTruncate,
    onBranch,
    sendDisabled = false,
    sendDisabledHint,
    modelLabel = null,
    isLoading = $bindable(false),
    placeholder = 'Type your message...',
    title = 'Chat',
    class: className = '',
    conversationId,
    showSelectors = true,
    selectedProvider = $bindable<ProviderType | null>(null),
    selectedBackendProvider = $bindable<string | null>(null),
    selectedModel = $bindable<string | null>(null),
    contextUsage = null,
    onOpenSettings,
    settingsOpen = false,
  }: Props = $props();

  let messageInput = $state('');
  let messagesContainer: HTMLElement | null = $state(null);
  let scrollViewport: HTMLElement | null = $state(null);
  let copiedIndex = $state<number | null>(null);

  async function copyMessage(index: number, content: string) {
    await navigator.clipboard.writeText(content);
    copiedIndex = index;
    setTimeout(() => {
      if (copiedIndex === index) copiedIndex = null;
    }, 1500);
  }

  /** Put a user turn back in the composer and drop it plus everything after. */
  async function editMessage(index: number, content: string) {
    messageInput = content;
    await onTruncate?.(index);
  }

  async function handleSend() {
    if (!messageInput.trim() || isLoading || sendDisabled) return;

    const currentMessage = messageInput.trim();
    messageInput = '';

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
        role: 'user',
        content: currentMessage,
        timestamp: new Date(),
      };
      messages = [...messages, userMessage];

      isLoading = true;
      try {
        const response = await aiChatService.sendMessage(currentMessage, messages, {
          provider: selectedProvider || undefined,
          llm_provider: selectedBackendProvider || undefined,
          conversation_id: conversationId,
          model: selectedModel || undefined,
        });
        const assistantMessage: ChatMessageType = {
          role: 'assistant',
          content: response,
          timestamp: new Date(),
        };
        messages = [...messages, assistantMessage];
      } catch (error) {
        console.error('Failed to send message:', error);
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

<div class="flex h-full min-h-0 flex-col {className}">
  {#if showSelectors}
    <div
      class="divider-edge-b divider-edge-full flex items-center justify-between gap-2 px-4 py-2.5"
    >
      <h2 class="flex items-center gap-2 text-sm font-semibold">
        <MessageSquare class="h-4 w-4" />
        {title}
      </h2>
      <ChatCatalogSelectors bind:selectedProvider bind:selectedBackendProvider bind:selectedModel />
    </div>
  {/if}
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
    <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollViewport}>
      <div
        class="mx-auto w-full max-w-chat space-y-6 px-[var(--content-gutter)] py-6"
        bind:this={messagesContainer}
      >
        {#if messages.length === 0}
          <div
            class="flex flex-col items-center justify-center py-24 text-center text-muted-foreground"
          >
            <MessageCircle class="mb-3 h-8 w-8 opacity-40" />
            <p class="text-sm">Start a conversation</p>
          </div>
        {:else}
          {#each messages as message, index}
            {@const isLastMessage = index === messages.length - 1}
            {@const shouldShowLoader =
              isLoading && isLastMessage && message.role === 'assistant' && !message.content}
            {@const isStreamingMessage =
              isLoading && isLastMessage && message.role === 'assistant' && !!message.content}
            <ChatMessage
              {message}
              {modelLabel}
              showLoader={shouldShowLoader}
              isStreaming={isStreamingMessage}
            >
              {#if message.content && !isStreamingMessage && !shouldShowLoader}
                <div
                  class="flex items-center gap-0.5 opacity-60 transition-opacity hover:opacity-100 focus-within:opacity-100 {message.role ===
                  'user'
                    ? 'justify-end'
                    : 'justify-start'}"
                >
                  <Button
                    size="icon"
                    variant="ghost"
                    class="h-6 w-6 text-muted-foreground"
                    title={copiedIndex === index ? 'Copied' : 'Copy'}
                    onclick={() => copyMessage(index, message.content)}
                  >
                    {#if copiedIndex === index}
                      <Check class="h-3 w-3" />
                    {:else}
                      <Copy class="h-3 w-3" />
                    {/if}
                  </Button>
                  {#if onRegenerate && isLastMessage && message.role === 'assistant'}
                    <Button
                      size="icon"
                      variant="ghost"
                      class="h-6 w-6 text-muted-foreground"
                      title={sendDisabled ? 'Unavailable while offline' : 'Regenerate'}
                      disabled={isLoading || sendDisabled}
                      onclick={() => onRegenerate?.()}
                    >
                      <RefreshCw class="h-3 w-3" />
                    </Button>
                  {/if}
                  {#if onBranch}
                    <Button
                      size="icon"
                      variant="ghost"
                      class="h-6 w-6 text-muted-foreground"
                      title="Branch a new conversation from here"
                      disabled={isLoading}
                      onclick={() => onBranch?.(index)}
                    >
                      <GitBranch class="h-3 w-3" />
                    </Button>
                  {/if}
                  {#if onTruncate && message.role === 'user'}
                    <Button
                      size="icon"
                      variant="ghost"
                      class="h-6 w-6 text-muted-foreground"
                      title="Edit and resend"
                      disabled={isLoading}
                      onclick={() => editMessage(index, message.content)}
                    >
                      <Pencil class="h-3 w-3" />
                    </Button>
                  {/if}
                  {#if onTruncate}
                    <Button
                      size="icon"
                      variant="ghost"
                      class="h-6 w-6 text-muted-foreground hover:text-destructive"
                      title="Delete this and everything after"
                      disabled={isLoading}
                      onclick={() => onTruncate?.(index)}
                    >
                      <Trash2 class="h-3 w-3" />
                    </Button>
                  {/if}
                </div>
              {/if}
            </ChatMessage>
          {/each}
        {/if}
      </div>
    </ScrollArea>
    <div class="shrink-0 bg-gradient-to-t from-background via-background to-transparent pt-2">
      <ChatInput
        bind:value={messageInput}
        onSend={handleSend}
        {onStop}
        running={isLoading}
        {placeholder}
        disabled={isLoading || sendDisabled}
        hint={sendDisabled ? sendDisabledHint : undefined}
        {contextUsage}
        {onOpenSettings}
        {settingsOpen}
        class="pt-0"
      />
    </div>
  </div>
</div>
