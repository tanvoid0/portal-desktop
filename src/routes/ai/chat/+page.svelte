<script lang="ts">
  import { onMount } from 'svelte';
  import { replaceState } from '$app/navigation';
  import { page } from '$app/stores';
  import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';
  import ConversationList from '$lib/domains/ai/components/conversations/ConversationList.svelte';
  import ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { aiConversationService, aiChatService, aiProviderService } from '$lib/domains/ai';
  import type {
    Conversation,
    CatalogStatus,
    ChatMessage,
    ContextUsage,
    ConversationMessage,
    LlmUsage,
    ProviderType,
  } from '$lib/domains/ai/types/index.js';
  import AIContextBar from '$lib/domains/ai/components/AIContextBar.svelte';
  import {
    fallbackTitleFromMessage,
    isPlaceholderTitle,
    reconcileThreadTitle,
  } from '$lib/domains/chat/title.js';
  import { PanelLeftOpen, Settings2, RefreshCw, ServerOff } from '@lucide/svelte';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import { Button } from '$lib/components/ui/button';
  import ChatSettingsPanel from '$lib/domains/ai/components/chat/ChatSettingsPanel.svelte';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';
  import {
    loadChatSettings,
    samplingExtras,
    type ChatSettings,
  } from '$lib/domains/ai/utils/chatSettings.js';

  let messages = $state<ChatMessage[]>([]);
  let isLoading = $state(false);
  let conversations = $state<Conversation[]>([]);
  let selectedConversation = $state<Conversation | null>(null);
  let selectedProvider = $state<ProviderType | null>(null);
  let selectedBackendProvider = $state<string | null>(null);
  let selectedModel = $state<string | null>(null);
  let isSending = $state(false);
  let conversationId = $state<string | undefined>(undefined);
  /** Id of the in-flight stream, so the stop button can cancel it. */
  let activeStreamId = $state<string | null>(null);
  /** Conversation ids with user-edited titles — ignore smart title events. */
  let userRenamedConversationIds = $state<Set<string>>(new Set());
  let conversationsPanelOpen = $state(false);
  let settingsPanelOpen = $state(false);
  let chatSettings = $state<ChatSettings>(loadChatSettings());
  let contextUsage = $state<ContextUsage | null>(null);
  let llmUsage = $state<LlmUsage | null>(null);
  let providerSelector = $state<ProviderModelSelector | null>(null);
  // Assume reachable until the first catalog load says otherwise.
  let platformOnline = $state(true);
  let platformChecking = $state(true);
  let platformError = $state<string | null>(null);
  const platformOffline = $derived(!platformOnline);

  /**
   * Keep the last completed verdict while a re-check is in flight, so the
   * offline banner stays mounted and can show its own progress.
   */
  function applyPlatformStatus(status: CatalogStatus) {
    platformChecking = status.checking;
    if (status.checking) return;
    platformOnline = status.online;
    platformError = status.error;
  }

  // Same deal as the coder page: park the controls in the AI layout's tab row.
  $effect(() => {
    aiTopbar.actions = topbarActions;
    return () => {
      aiTopbar.actions = null;
    };
  });

  function patchConversationTitle(id: string, title: string) {
    if (userRenamedConversationIds.has(id)) return;

    const idx = conversations.findIndex((c) => c.id === id);
    const current = idx >= 0 ? conversations[idx].title : selectedConversation?.title;
    const next = reconcileThreadTitle(current, title);

    if (idx >= 0) {
      conversations[idx] = { ...conversations[idx], title: next };
      conversations = [...conversations];
    }
    if (selectedConversation?.id === id) {
      selectedConversation = { ...selectedConversation, title: next };
    }
  }

  onMount(async () => {
    const defaultProvider = await aiProviderService.getDefaultProvider();
    selectedProvider = defaultProvider || 'AgentPlatform';

    // Load conversations first
    await loadConversations();

    // Check for conversation ID in URL query params after conversations are loaded
    const urlConversationId = $page.url.searchParams.get('id');
    if (urlConversationId) {
      const conversation = conversations.find((c) => c.id === urlConversationId);
      if (conversation) {
        await handleConversationClick(conversation);
      } else {
        // Try loading the conversation directly if not in list
        try {
          const result = await aiConversationService.loadConversation(urlConversationId);
          applyConversationSelection(result.conversation);
          messages = result.messages.map((msg: ConversationMessage) => ({
            role: msg.role,
            content: msg.content,
            timestamp: new Date(msg.timestamp),
          }));
          conversationId = urlConversationId;
          await loadConversations(); // Reload to include it in the list
        } catch (error) {
          toastActions.error('Failed to load conversation from URL', error);
        }
      }
    }
  });

  async function loadConversations() {
    try {
      conversations = await aiConversationService.listConversations();
    } catch (error) {
      toastActions.error('Failed to load conversations', error);
    }
  }

  function applyConversationSelection(conversation: Conversation) {
    // Usage is reported per turn — it does not carry across threads.
    contextUsage = null;
    llmUsage = null;
    selectedConversation = conversation;
    selectedProvider = conversation.provider;
    selectedModel = conversation.model ?? null;
  }

  async function handleConversationClick(conversation: Conversation) {
    if (selectedConversation?.id === conversation.id) return;

    isLoading = true;
    try {
      const result = await aiConversationService.loadConversation(conversation.id);
      applyConversationSelection(result.conversation);
      messages = result.messages.map((msg: ConversationMessage) => ({
        role: msg.role,
        content: msg.content,
        timestamp: new Date(msg.timestamp),
      }));
      conversationId = conversation.id;

      // Update URL without navigation
      const url = new URL($page.url);
      url.searchParams.set('id', conversation.id);
      replaceState(url, {});
    } catch (error) {
      toastActions.error('Failed to load conversation', error);
    } finally {
      isLoading = false;
    }
  }

  async function handleDeleteConversation(conversation: Conversation) {
    const confirmed = await confirmAction(
      `Delete conversation "${conversation.title}"?`,
      'Delete conversation'
    );
    if (!confirmed) return;

    try {
      await aiConversationService.deleteConversation(conversation.id);
      toastActions.success('Conversation deleted');

      // If deleted conversation was selected, clear selection
      if (selectedConversation?.id === conversation.id) {
        handleNewConversation();
      }

      await loadConversations();
    } catch (error) {
      toastActions.error('Failed to delete conversation', error);
    }
  }

  async function handleDeleteAllConversations() {
    const confirmed = await confirmAction(
      `Delete all ${conversations.length} conversations? This cannot be undone.`,
      'Delete all conversations'
    );
    if (!confirmed) return;

    try {
      // Delete all conversations one by one
      const deletePromises = conversations.map((conv) =>
        aiConversationService.deleteConversation(conv.id)
      );
      await Promise.all(deletePromises);

      toastActions.success(`Deleted ${conversations.length} conversations`);

      // Clear selection
      handleNewConversation();
      await loadConversations();
    } catch (error) {
      toastActions.error('Failed to delete all conversations', error);
    }
  }

  function handleNewConversation() {
    selectedConversation = null;
    messages = [];
    conversationId = undefined;
    contextUsage = null;
    llmUsage = null;

    // Clear URL parameter
    const url = new URL($page.url);
    url.searchParams.delete('id');
    replaceState(url, {});
  }

  async function handleModelChange(model: string) {
    if (!selectedConversation) return;
    try {
      await aiConversationService.updateConversationModel(selectedConversation.id, model);
      selectedConversation = { ...selectedConversation, model };
    } catch (error) {
      toastActions.error('Failed to update model', error);
    }
  }

  async function handleSendMessage(message: string, history: ChatMessage[]) {
    if (!message.trim() || isSending || platformOffline) return;

    const userMessage: ChatMessage = {
      role: 'user',
      content: message.trim(),
      timestamp: new Date(),
    };

    messages = [...messages, userMessage];
    isSending = true;

    // Create assistant message placeholder for streaming
    const assistantMessageIndex = messages.length;
    messages = [
      ...messages,
      {
        role: 'assistant',
        content: '',
        timestamp: new Date(),
      },
    ];

    try {
      if (selectedConversation && selectedModel && selectedModel !== selectedConversation.model) {
        await aiConversationService.updateConversationModel(selectedConversation.id, selectedModel);
        selectedConversation = { ...selectedConversation, model: selectedModel };
      }

      // First message on new thread: create with placeholder, optimistic fallback in sidebar
      if (!selectedConversation && selectedProvider) {
        const conversation = await aiConversationService.createConversation(
          'New chat',
          selectedProvider,
          selectedModel
        );
        selectedConversation = conversation;
        conversationId = conversation.id;
        conversations = [conversation, ...conversations];

        const fb = fallbackTitleFromMessage(message.trim(), 'New chat');
        patchConversationTitle(conversation.id, fb);
      } else if (
        selectedConversation &&
        isPlaceholderTitle(selectedConversation.title) &&
        history.filter((m) => m.role === 'user').length === 0
      ) {
        const fb = fallbackTitleFromMessage(message.trim(), 'New chat');
        patchConversationTitle(selectedConversation.id, fb);
      }

      const streamFallback = fallbackTitleFromMessage(message.trim(), 'New chat');

      // Timings for the LM Studio-style stat chips under the reply.
      const startedAt = performance.now();
      let thinkingMs: number | null = null;

      // Use streaming API
      const fullResponse = await aiChatService.streamMessage(message, history, {
        provider: selectedConversation?.provider || selectedProvider || undefined,
        llm_provider: selectedBackendProvider || undefined,
        conversation_id: conversationId,
        model: selectedModel || undefined,
        system_prompt: chatSettings.systemPrompt || undefined,
        temperature: chatSettings.temperature,
        max_tokens: chatSettings.limitResponseLength ? chatSettings.maxTokens : undefined,
        extra_options: samplingExtras(chatSettings),
        onStreamId: (id) => (activeStreamId = id),
        onTitleUpdated: ({ conversation_id, title }) => {
          patchConversationTitle(conversation_id, title);
        },
        onChunk: (chunk: string) => {
          messages[assistantMessageIndex].content += chunk;
          if (thinkingMs === null && messages[assistantMessageIndex].content.includes('</think>')) {
            thinkingMs = performance.now() - startedAt;
          }
          messages = [...messages];
        },
        onComplete: (fullMessage: string, payload) => {
          const durationMs = performance.now() - startedAt;
          const completionTokens = payload?.llm_usage?.completion_tokens ?? null;
          messages[assistantMessageIndex].content = fullMessage;
          messages[assistantMessageIndex].stats = {
            completionTokens,
            durationMs,
            thinkingMs,
            tokensPerSecond:
              completionTokens && durationMs > 0 ? completionTokens / (durationMs / 1000) : null,
            stopReason: payload?.cancelled ? 'Stopped by user' : (payload?.finish_reason ?? null),
          };
          messages = [...messages];
          if (payload?.context_usage) contextUsage = payload.context_usage;
          if (payload?.llm_usage) llmUsage = payload.llm_usage;
          if (payload?.title && conversationId) {
            patchConversationTitle(
              conversationId,
              reconcileThreadTitle(selectedConversation?.title, payload.title, streamFallback)
            );
          }
        },
      });

      // Ensure final message is set
      messages[assistantMessageIndex].content = fullResponse;
      messages = [...messages];

      // Stopped before the first token — drop the empty assistant bubble.
      if (!fullResponse) {
        messages = messages.slice(0, assistantMessageIndex);
      }

      // Save conversation if we have one
      if (selectedConversation) {
        await persistMessages();
        await loadConversations();

        // Update URL if not already set
        if (!conversationId) {
          const url = new URL($page.url);
          url.searchParams.set('id', selectedConversation.id);
          replaceState(url, {});
        }
      }
    } catch (error) {
      toastActions.error('Failed to send message', error);
      // Remove both user and assistant messages on error
      messages = messages.slice(0, -2);
      // The send may have failed because the platform went away — re-check so
      // the banner appears instead of letting the next send fail the same way.
      void retryPlatform();
    } finally {
      isSending = false;
      activeStreamId = null;
    }
  }

  function toWireMessages(id: string, list: ChatMessage[]) {
    return list.map((msg, idx) => ({
      id: `${id}-${idx}`,
      conversation_id: id,
      role: msg.role,
      content: msg.content,
      timestamp:
        msg.timestamp instanceof Date
          ? msg.timestamp.toISOString()
          : msg.timestamp || new Date().toISOString(),
      sequence: idx,
    }));
  }

  /** Persist the current message array over the stored conversation. */
  async function persistMessages() {
    if (!selectedConversation) return;
    const id = selectedConversation.id;
    await aiConversationService.saveConversation(id, toWireMessages(id, messages));
  }

  /**
   * Fork the thread: everything up to and including `index` becomes a new
   * conversation, which then takes over the view. The original is untouched.
   */
  async function handleBranch(index: number) {
    if (isSending) return;
    const provider = selectedConversation?.provider ?? selectedProvider;
    if (!provider) return;

    const branched = messages.slice(0, index + 1);
    try {
      const conversation = await aiConversationService.createConversation(
        selectedConversation?.title ?? 'New chat',
        provider,
        selectedConversation?.model ?? selectedModel
      );
      await aiConversationService.saveConversation(
        conversation.id,
        toWireMessages(conversation.id, branched)
      );

      applyConversationSelection(conversation);
      messages = branched;
      conversationId = conversation.id;
      await loadConversations();

      const url = new URL($page.url);
      url.searchParams.set('id', conversation.id);
      replaceState(url, {});
      toastActions.success('Branched into a new conversation');
    } catch (error) {
      toastActions.error('Failed to branch conversation', error);
    }
  }

  async function handleTruncate(index: number) {
    if (isSending) return;
    messages = messages.slice(0, index);
    try {
      await persistMessages();
      await loadConversations();
    } catch (error) {
      toastActions.error('Failed to update conversation', error);
    }
  }

  async function handleRegenerate() {
    if (isSending) return;
    const lastUserIndex = messages.map((m) => m.role).lastIndexOf('user');
    if (lastUserIndex < 0) return;

    const prompt = messages[lastUserIndex].content;
    const history = messages.slice(0, lastUserIndex);
    messages = history;
    await handleSendMessage(prompt, history);
  }

  /** Re-fetch the catalog; that round trip is the reachability check. */
  async function retryPlatform() {
    await providerSelector?.reload();
  }

  async function handleStop() {
    if (!activeStreamId) return;
    try {
      await aiChatService.cancelStream(activeStreamId);
    } catch (error) {
      toastActions.error('Failed to stop generation', error);
    }
  }
</script>

{#snippet topbarActions()}
  <AIContextBar {contextUsage} {llmUsage} variant="ring" />
  <ProviderModelSelector
    bind:this={providerSelector}
    bind:selectedProvider
    bind:selectedBackendProvider
    bind:selectedModel
    onModelChange={handleModelChange}
    onStatusChange={applyPlatformStatus}
    showInlineError={false}
    backendSelectClass="w-[130px]"
    modelSelectClass="w-[220px]"
  />
  <Button
    size="icon"
    variant={settingsPanelOpen ? 'secondary' : 'ghost'}
    class="h-8 w-8 shrink-0"
    title="Chat settings"
    onclick={() => (settingsPanelOpen = !settingsPanelOpen)}
  >
    <Settings2 class="h-4 w-4" />
  </Button>
{/snippet}

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel bind:open={conversationsPanelOpen} side="left" desktopClass="w-64">
    <ConversationList
      bind:conversations
      onConversationClick={(c) => {
        handleConversationClick(c);
        conversationsPanelOpen = false;
      }}
      onCreateNew={handleNewConversation}
      onDeleteConversation={handleDeleteConversation}
      onDeleteAll={handleDeleteAllConversations}
      selectedConversationId={selectedConversation?.id}
    />
  </ResponsivePanel>

  <!-- Main Chat Area -->
  <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <div
      class="divider-edge-b divider-edge-full flex flex-wrap items-center justify-between gap-2 bg-background px-4 py-2.5"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8 shrink-0 md:hidden"
          title="Conversations"
          onclick={() => (conversationsPanelOpen = true)}
        >
          <PanelLeftOpen class="h-4 w-4" />
        </Button>
        <h2 class="truncate text-sm font-semibold">
          {selectedConversation?.title || 'AI Chat'}
        </h2>
      </div>
    </div>
    {#if platformOffline}
      <div
        class="divider-edge-b divider-edge-full flex flex-wrap items-center gap-3 bg-destructive/10 px-4 py-2.5 text-destructive"
        role="status"
      >
        <ServerOff class="h-4 w-4 shrink-0" />
        <div class="min-w-0 flex-1">
          <p class="text-xs font-medium">Agent Platform is unreachable</p>
          <p class="truncate text-[11px] opacity-80" title={platformError ?? ''}>
            {platformError ?? 'Check that the platform is running.'}
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          class="h-7 shrink-0 gap-1.5 text-xs"
          disabled={platformChecking}
          onclick={retryPlatform}
        >
          <RefreshCw class="h-3 w-3 {platformChecking ? 'animate-spin' : ''}" />
          {platformChecking ? 'Checking…' : 'Retry'}
        </Button>
      </div>
    {/if}
    <div class="flex min-h-0 w-full flex-1 flex-col">
      <AIChatPanel
        bind:messages
        isLoading={isSending || isLoading}
        title={selectedConversation?.title || 'AI Chat'}
        placeholder="Ask me anything..."
        modelLabel={selectedModel}
        {conversationId}
        showSelectors={false}
        {contextUsage}
        settingsOpen={settingsPanelOpen}
        onOpenSettings={() => (settingsPanelOpen = !settingsPanelOpen)}
        onSendMessageWithHistory={handleSendMessage}
        onStop={handleStop}
        onRegenerate={handleRegenerate}
        onTruncate={handleTruncate}
        onBranch={handleBranch}
        sendDisabled={platformOffline}
        sendDisabledHint="Agent Platform is offline — retry to reconnect."
      />
    </div>
  </main>

  {#if settingsPanelOpen}
    <ResponsivePanel bind:open={settingsPanelOpen} side="right" desktopClass="w-72">
      {#snippet header()}
        <div class="divider-edge-b divider-edge-full p-2.5">
          <h2 class="flex items-center gap-1.5 text-sm font-semibold">
            <Settings2 class="h-4 w-4" />
            Model Parameters
          </h2>
        </div>
      {/snippet}
      <ChatSettingsPanel bind:settings={chatSettings} />
    </ResponsivePanel>
  {/if}
</div>
