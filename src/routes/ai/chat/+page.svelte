<script lang="ts">
  import { onMount } from "svelte";
  import { replaceState } from "$app/navigation";
  import { page } from "$app/stores";
  import AIChatPanel from "$lib/domains/ai/components/chat/AIChatPanel.svelte";
  import ConversationList from "$lib/domains/ai/components/conversations/ConversationList.svelte";
  import ProviderModelSelector from "$lib/domains/ai/components/ProviderModelSelector.svelte";
  import { toastActions } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import {
    aiConversationService,
    aiChatService,
    aiProviderService,
  } from "$lib/domains/ai";
  import type {
    Conversation,
    ChatMessage,
    ConversationMessage,
    ProviderType,
  } from "$lib/domains/ai/types/index.js";
  import {
    fallbackTitleFromMessage,
    isPlaceholderTitle,
    reconcileThreadTitle,
  } from "$lib/domains/chat/title.js";
  import { MessageSquare, PanelLeftOpen } from "@lucide/svelte";
  import ResponsivePanel from "$lib/components/shell/responsive-panel.svelte";
  import { Button } from "$lib/components/ui/button";

  let messages = $state<ChatMessage[]>([]);
  let isLoading = $state(false);
  let conversations = $state<Conversation[]>([]);
  let selectedConversation = $state<Conversation | null>(null);
  let selectedProvider = $state<ProviderType | null>(null);
  let selectedBackendProvider = $state<string | null>(null);
  let selectedModel = $state<string | null>(null);
  let isSending = $state(false);
  let conversationId = $state<string | undefined>(undefined);
  /** Conversation ids with user-edited titles — ignore smart title events. */
  let userRenamedConversationIds = $state<Set<string>>(new Set());
  let conversationsPanelOpen = $state(false);

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
    selectedProvider = defaultProvider || "AgentPlatform";

    // Load conversations first
    await loadConversations();

    // Check for conversation ID in URL query params after conversations are loaded
    const urlConversationId = $page.url.searchParams.get("id");
    if (urlConversationId) {
      const conversation = conversations.find(
        (c) => c.id === urlConversationId,
      );
      if (conversation) {
        await handleConversationClick(conversation);
      } else {
        // Try loading the conversation directly if not in list
        try {
          const result =
            await aiConversationService.loadConversation(urlConversationId);
          applyConversationSelection(result.conversation);
          messages = result.messages.map((msg: ConversationMessage) => ({
            role: msg.role,
            content: msg.content,
            timestamp: new Date(msg.timestamp),
          }));
          conversationId = urlConversationId;
          await loadConversations(); // Reload to include it in the list
        } catch (error) {
          toastActions.error("Failed to load conversation from URL", error);
        }
      }
    }
  });

  async function loadConversations() {
    try {
      conversations = await aiConversationService.listConversations();
    } catch (error) {
      toastActions.error("Failed to load conversations", error);
    }
  }

  function applyConversationSelection(conversation: Conversation) {
    selectedConversation = conversation;
    selectedProvider = conversation.provider;
    selectedModel = conversation.model ?? null;
  }

  async function handleConversationClick(conversation: Conversation) {
    if (selectedConversation?.id === conversation.id) return;

    isLoading = true;
    try {
      const result = await aiConversationService.loadConversation(
        conversation.id,
      );
      applyConversationSelection(result.conversation);
      messages = result.messages.map((msg: ConversationMessage) => ({
        role: msg.role,
        content: msg.content,
        timestamp: new Date(msg.timestamp),
      }));
      conversationId = conversation.id;

      // Update URL without navigation
      const url = new URL($page.url);
      url.searchParams.set("id", conversation.id);
      replaceState(url, {});
    } catch (error) {
      toastActions.error("Failed to load conversation", error);
    } finally {
      isLoading = false;
    }
  }

  async function handleDeleteConversation(conversation: Conversation) {
    const confirmed = await confirmAction(
      `Delete conversation "${conversation.title}"?`,
      "Delete conversation",
    );
    if (!confirmed) return;

    try {
      await aiConversationService.deleteConversation(conversation.id);
      toastActions.success("Conversation deleted");

      // If deleted conversation was selected, clear selection
      if (selectedConversation?.id === conversation.id) {
        handleNewConversation();
      }

      await loadConversations();
    } catch (error) {
      toastActions.error("Failed to delete conversation", error);
    }
  }

  async function handleDeleteAllConversations() {
    const confirmed = await confirmAction(
      `Delete all ${conversations.length} conversations? This cannot be undone.`,
      "Delete all conversations",
    );
    if (!confirmed) return;

    try {
      // Delete all conversations one by one
      const deletePromises = conversations.map((conv) =>
        aiConversationService.deleteConversation(conv.id),
      );
      await Promise.all(deletePromises);

      toastActions.success(`Deleted ${conversations.length} conversations`);

      // Clear selection
      handleNewConversation();
      await loadConversations();
    } catch (error) {
      toastActions.error("Failed to delete all conversations", error);
    }
  }

  function handleNewConversation() {
    selectedConversation = null;
    messages = [];
    conversationId = undefined;

    // Clear URL parameter
    const url = new URL($page.url);
    url.searchParams.delete("id");
    replaceState(url, {});
  }

  async function handleModelChange(model: string) {
    if (!selectedConversation) return;
    try {
      await aiConversationService.updateConversationModel(
        selectedConversation.id,
        model,
      );
      selectedConversation = { ...selectedConversation, model };
    } catch (error) {
      toastActions.error("Failed to update model", error);
    }
  }

  async function handleSendMessage(message: string, history: ChatMessage[]) {
    if (!message.trim() || isSending) return;

    const userMessage: ChatMessage = {
      role: "user",
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
        role: "assistant",
        content: "",
        timestamp: new Date(),
      },
    ];

    try {
      if (
        selectedConversation &&
        selectedModel &&
        selectedModel !== selectedConversation.model
      ) {
        await aiConversationService.updateConversationModel(
          selectedConversation.id,
          selectedModel,
        );
        selectedConversation = { ...selectedConversation, model: selectedModel };
      }

      // First message on new thread: create with placeholder, optimistic fallback in sidebar
      if (!selectedConversation && selectedProvider) {
        const conversation = await aiConversationService.createConversation(
          "New chat",
          selectedProvider,
          selectedModel,
        );
        selectedConversation = conversation;
        conversationId = conversation.id;
        conversations = [conversation, ...conversations];

        const fb = fallbackTitleFromMessage(message.trim(), "New chat");
        patchConversationTitle(conversation.id, fb);
      } else if (
        selectedConversation &&
        isPlaceholderTitle(selectedConversation.title) &&
        history.filter((m) => m.role === "user").length === 0
      ) {
        const fb = fallbackTitleFromMessage(message.trim(), "New chat");
        patchConversationTitle(selectedConversation.id, fb);
      }

      const streamFallback = fallbackTitleFromMessage(message.trim(), "New chat");

      // Use streaming API
      const fullResponse = await aiChatService.streamMessage(message, history, {
        provider:
          selectedConversation?.provider || selectedProvider || undefined,
        llm_provider: selectedBackendProvider || undefined,
        conversation_id: conversationId,
        model: selectedModel || undefined,
        onTitleUpdated: ({ conversation_id, title }) => {
          patchConversationTitle(conversation_id, title);
        },
        onChunk: (chunk: string) => {
          messages[assistantMessageIndex].content += chunk;
          messages = [...messages];
        },
        onComplete: (fullMessage: string, payload) => {
          messages[assistantMessageIndex].content = fullMessage;
          messages = [...messages];
          if (payload?.title && conversationId) {
            patchConversationTitle(
              conversationId,
              reconcileThreadTitle(
                selectedConversation?.title,
                payload.title,
                streamFallback,
              ),
            );
          }
        },
      });

      // Ensure final message is set
      messages[assistantMessageIndex].content = fullResponse;
      messages = [...messages];

      // Save conversation if we have one
      if (selectedConversation) {
        await aiConversationService.saveConversation(selectedConversation.id, [
          ...messages.map((msg, idx) => ({
            id: `${selectedConversation!.id}-${idx}`,
            conversation_id: selectedConversation!.id,
            role: msg.role,
            content: msg.content,
            timestamp: msg.timestamp?.toISOString() || new Date().toISOString(),
            sequence: idx,
          })),
        ]);
        await loadConversations();

        // Update URL if not already set
        if (!conversationId) {
          const url = new URL($page.url);
          url.searchParams.set("id", selectedConversation.id);
          replaceState(url, {});
        }
      }
    } catch (error) {
      toastActions.error("Failed to send message", error);
      // Remove both user and assistant messages on error
      messages = messages.slice(0, -2);
    } finally {
      isSending = false;
    }
  }
</script>

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel
    bind:open={conversationsPanelOpen}
    side="left"
    desktopClass="w-64"
  >
    {#snippet header()}
      <div class="border-b p-2.5">
        <h2 class="flex items-center gap-1.5 text-sm font-semibold">
          <MessageSquare class="h-4 w-4" />
          Conversations
        </h2>
      </div>
    {/snippet}
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
      class="flex flex-wrap items-center justify-between gap-2 border-b bg-background px-4 py-2.5"
    >
      <div class="flex min-w-0 items-center gap-2">
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
          {selectedConversation?.title || "AI Chat"}
        </h2>
      </div>
      <ProviderModelSelector
        bind:selectedProvider
        bind:selectedBackendProvider
        bind:selectedModel
        onModelChange={handleModelChange}
        backendSelectClass="w-[130px]"
        modelSelectClass="w-[220px]"
      />
    </div>
    <div class="flex h-full min-h-0 w-full flex-col p-6">
      <AIChatPanel
        bind:messages
        isLoading={isSending || isLoading}
        title={selectedConversation?.title || "AI Chat"}
        placeholder="Ask me anything..."
        class="flex h-full flex-col border-0 shadow-none"
        {conversationId}
        showSelectors={false}
        onSendMessageWithHistory={handleSendMessage}
      />
    </div>
  </main>
</div>
