import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import type ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
import { toastActions } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import { aiConversationService, aiChatService, aiProviderService } from '$lib/domains/ai';
import type {
  Conversation,
  CatalogStatus,
  ChatMessage,
  ContextUsage,
  LlmUsage,
  ProviderType,
} from '$lib/domains/ai/types/index.js';
import { isAuthError } from '$lib/domains/ai/utils/catalog.js';
import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
import {
  fromConversationMessages,
  toWireMessages,
} from '$lib/domains/ai/utils/conversationWire.js';
import {
  fallbackTitleFromMessage,
  isPlaceholderTitle,
  reconcileThreadTitle,
} from '$lib/domains/chat/title.js';
import {
  loadChatSettings,
  samplingExtras,
  type ChatSettings,
} from '$lib/domains/ai/utils/chatSettings.js';

/** State and handlers behind the /ai/chat page. */
export class AIChatPage {
  messages = $state<ChatMessage[]>([]);
  isLoading = $state(false);
  conversations = $state<Conversation[]>([]);
  selectedConversation = $state<Conversation | null>(null);
  selectedProvider = $state<ProviderType | null>(null);
  selectedBackendProvider = $state<string | null>(null);
  selectedModel = $state<string | null>(null);
  isSending = $state(false);
  conversationId = $state<string | undefined>(undefined);
  /** Id of the in-flight stream, so the stop button can cancel it. */
  activeStreamId = $state<string | null>(null);
  /** Conversation ids with user-edited titles — ignore smart title events. */
  userRenamedConversationIds = new Set<string>();
  conversationsPanelOpen = $state(false);
  settingsPanelOpen = $state(false);
  chatSettings = $state<ChatSettings>(loadChatSettings());
  contextUsage = $state<ContextUsage | null>(null);
  llmUsage = $state<LlmUsage | null>(null);
  // Assume reachable until the first catalog load says otherwise.
  platformOnline = $state(true);
  platformChecking = $state(true);
  platformError = $state<string | null>(null);
  // The health probe is authoritative when it has a verdict; the catalog
  // round trip still catches failures the probe cannot see.
  platformOffline = $derived(!this.platformOnline || aiPlatform.blocked);
  platformUnauthorized = $derived(
    aiPlatform.blocked
      ? aiPlatform.status === 'unauthenticated'
      : this.platformOffline && isAuthError(this.platformError)
  );

  constructor(private getSelector: () => ProviderModelSelector | null) {}

  /**
   * Keep the last completed verdict while a re-check is in flight, so the
   * offline banner stays mounted and can show its own progress.
   */
  applyPlatformStatus = (status: CatalogStatus) => {
    this.platformChecking = status.checking;
    if (status.checking) return;
    this.platformOnline = status.online;
    this.platformError = status.error;
  };

  /** Mirror the open conversation in the URL (?id=) without navigating. */
  private setUrlConversationId(id: string | null) {
    const url = new URL(page.url.href);
    if (id) url.searchParams.set('id', id);
    else url.searchParams.delete('id');
    replaceState(url, {});
  }

  private patchConversationTitle(id: string, title: string) {
    if (this.userRenamedConversationIds.has(id)) return;

    const idx = this.conversations.findIndex((c) => c.id === id);
    const current = idx >= 0 ? this.conversations[idx].title : this.selectedConversation?.title;
    const next = reconcileThreadTitle(current, title);

    if (idx >= 0) {
      this.conversations[idx] = { ...this.conversations[idx], title: next };
      this.conversations = [...this.conversations];
    }
    if (this.selectedConversation?.id === id) {
      this.selectedConversation = { ...this.selectedConversation, title: next };
    }
  }

  init = async () => {
    void aiPlatform.refresh();
    const defaultProvider = await aiProviderService.getDefaultProvider();
    this.selectedProvider = defaultProvider || 'AgentPlatform';

    // Load conversations first
    await this.loadConversations();

    // Check for conversation ID in URL query params after conversations are loaded
    const urlConversationId = page.url.searchParams.get('id');
    if (!urlConversationId) return;
    const conversation = this.conversations.find((c) => c.id === urlConversationId);
    if (conversation) {
      await this.handleConversationClick(conversation);
      return;
    }
    // Try loading the conversation directly if not in list
    try {
      const result = await aiConversationService.loadConversation(urlConversationId);
      this.applyConversationSelection(result.conversation);
      this.messages = fromConversationMessages(result.messages);
      this.conversationId = urlConversationId;
      await this.loadConversations(); // Reload to include it in the list
    } catch (error) {
      toastActions.error('Failed to load conversation from URL', error);
    }
  };

  private async loadConversations() {
    try {
      this.conversations = await aiConversationService.listConversations();
    } catch (error) {
      toastActions.error('Failed to load conversations', error);
    }
  }

  private applyConversationSelection(conversation: Conversation) {
    // Usage is reported per turn — it does not carry across threads.
    this.contextUsage = null;
    this.llmUsage = null;
    this.selectedConversation = conversation;
    this.selectedProvider = conversation.provider;
    this.selectedModel = conversation.model ?? null;
  }

  handleConversationClick = async (conversation: Conversation) => {
    if (this.selectedConversation?.id === conversation.id) return;

    this.isLoading = true;
    try {
      const result = await aiConversationService.loadConversation(conversation.id);
      this.applyConversationSelection(result.conversation);
      this.messages = fromConversationMessages(result.messages);
      this.conversationId = conversation.id;

      // Update URL without navigation
      this.setUrlConversationId(conversation.id);
    } catch (error) {
      toastActions.error('Failed to load conversation', error);
    } finally {
      this.isLoading = false;
    }
  };

  handleDeleteConversation = async (conversation: Conversation) => {
    const confirmed = await confirmAction(
      `Delete conversation "${conversation.title}"?`,
      'Delete conversation'
    );
    if (!confirmed) return;

    try {
      await aiConversationService.deleteConversation(conversation.id);
      toastActions.success('Conversation deleted');

      // If deleted conversation was selected, clear selection
      if (this.selectedConversation?.id === conversation.id) {
        this.handleNewConversation();
      }

      await this.loadConversations();
    } catch (error) {
      toastActions.error('Failed to delete conversation', error);
    }
  };

  handleDeleteAllConversations = async () => {
    const confirmed = await confirmAction(
      `Delete all ${this.conversations.length} conversations? This cannot be undone.`,
      'Delete all conversations'
    );
    if (!confirmed) return;

    try {
      await Promise.all(
        this.conversations.map((conv) => aiConversationService.deleteConversation(conv.id))
      );

      toastActions.success(`Deleted ${this.conversations.length} conversations`);

      this.handleNewConversation();
      await this.loadConversations();
    } catch (error) {
      toastActions.error('Failed to delete all conversations', error);
    }
  };

  handleNewConversation = () => {
    this.selectedConversation = null;
    this.messages = [];
    this.conversationId = undefined;
    this.contextUsage = null;
    this.llmUsage = null;

    // Clear URL parameter
    this.setUrlConversationId(null);
  };

  handleModelChange = async (model: string) => {
    if (!this.selectedConversation) return;
    try {
      await aiConversationService.updateConversationModel(this.selectedConversation.id, model);
      this.selectedConversation = { ...this.selectedConversation, model };
    } catch (error) {
      toastActions.error('Failed to update model', error);
    }
  };

  handleSendMessage = async (message: string, history: ChatMessage[]) => {
    if (!message.trim() || this.isSending || this.platformOffline) return;

    const userMessage: ChatMessage = {
      role: 'user',
      content: message.trim(),
      timestamp: new Date(),
    };

    this.messages = [...this.messages, userMessage];
    this.isSending = true;

    // Create assistant message placeholder for streaming
    const assistantMessageIndex = this.messages.length;
    this.messages = [
      ...this.messages,
      {
        role: 'assistant',
        content: '',
        timestamp: new Date(),
      },
    ];

    try {
      if (
        this.selectedConversation &&
        this.selectedModel &&
        this.selectedModel !== this.selectedConversation.model
      ) {
        await aiConversationService.updateConversationModel(
          this.selectedConversation.id,
          this.selectedModel
        );
        this.selectedConversation = { ...this.selectedConversation, model: this.selectedModel };
      }

      // First message on new thread: create with placeholder, optimistic fallback in sidebar
      if (!this.selectedConversation && this.selectedProvider) {
        const conversation = await aiConversationService.createConversation(
          'New chat',
          this.selectedProvider,
          this.selectedModel
        );
        this.selectedConversation = conversation;
        this.conversationId = conversation.id;
        this.conversations = [conversation, ...this.conversations];

        const fb = fallbackTitleFromMessage(message.trim(), 'New chat');
        this.patchConversationTitle(conversation.id, fb);
      } else if (
        this.selectedConversation &&
        isPlaceholderTitle(this.selectedConversation.title) &&
        history.filter((m) => m.role === 'user').length === 0
      ) {
        const fb = fallbackTitleFromMessage(message.trim(), 'New chat');
        this.patchConversationTitle(this.selectedConversation.id, fb);
      }

      const streamFallback = fallbackTitleFromMessage(message.trim(), 'New chat');

      // Timings for the LM Studio-style stat chips under the reply.
      const startedAt = performance.now();
      let thinkingMs: number | null = null;
      const s = this.chatSettings;

      // Use streaming API
      const fullResponse = await aiChatService.streamMessage(message, history, {
        provider: this.selectedConversation?.provider || this.selectedProvider || undefined,
        llm_provider: this.selectedBackendProvider || undefined,
        conversation_id: this.conversationId,
        model: this.selectedModel || undefined,
        system_prompt: s.systemPrompt || undefined,
        temperature: s.temperature,
        max_tokens: s.limitResponseLength ? s.maxTokens : undefined,
        extra_options: samplingExtras(s),
        onStreamId: (id) => (this.activeStreamId = id),
        onTitleUpdated: ({ conversation_id, title }) => {
          this.patchConversationTitle(conversation_id, title);
        },
        onChunk: (chunk: string) => {
          this.messages[assistantMessageIndex].content += chunk;
          if (
            thinkingMs === null &&
            this.messages[assistantMessageIndex].content.includes('</think>')
          ) {
            thinkingMs = performance.now() - startedAt;
          }
          this.messages = [...this.messages];
        },
        onComplete: (fullMessage: string, payload) => {
          const durationMs = performance.now() - startedAt;
          const completionTokens = payload?.llm_usage?.completion_tokens ?? null;
          this.messages[assistantMessageIndex].content = fullMessage;
          this.messages[assistantMessageIndex].stats = {
            completionTokens,
            durationMs,
            thinkingMs,
            tokensPerSecond:
              completionTokens && durationMs > 0 ? completionTokens / (durationMs / 1000) : null,
            stopReason: payload?.cancelled ? 'Stopped by user' : (payload?.finish_reason ?? null),
          };
          this.messages = [...this.messages];
          if (payload?.context_usage) this.contextUsage = payload.context_usage;
          if (payload?.llm_usage) this.llmUsage = payload.llm_usage;
          if (payload?.title && this.conversationId) {
            this.patchConversationTitle(
              this.conversationId,
              reconcileThreadTitle(this.selectedConversation?.title, payload.title, streamFallback)
            );
          }
        },
      });

      // Ensure final message is set
      this.messages[assistantMessageIndex].content = fullResponse;
      this.messages = [...this.messages];

      // Stopped before the first token — drop the empty assistant bubble.
      if (!fullResponse) {
        this.messages = this.messages.slice(0, assistantMessageIndex);
      }

      // Save conversation if we have one
      if (this.selectedConversation) {
        await this.persistMessages();
        await this.loadConversations();

        // Update URL if not already set
        if (!this.conversationId) {
          this.setUrlConversationId(this.selectedConversation.id);
        }
      }
    } catch (error) {
      toastActions.error('Failed to send message', error);
      // Remove both user and assistant messages on error
      this.messages = this.messages.slice(0, -2);
      // The send may have failed because the platform went away — re-check so
      // the banner appears instead of letting the next send fail the same way.
      void this.retryPlatform();
    } finally {
      this.isSending = false;
      this.activeStreamId = null;
    }
  };

  /** Persist the current message array over the stored conversation. */
  private async persistMessages() {
    if (!this.selectedConversation) return;
    const id = this.selectedConversation.id;
    await aiConversationService.saveConversation(id, toWireMessages(id, this.messages));
  }

  /**
   * Fork the thread: everything up to and including `index` becomes a new
   * conversation, which then takes over the view. The original is untouched.
   */
  handleBranch = async (index: number) => {
    if (this.isSending) return;
    const provider = this.selectedConversation?.provider ?? this.selectedProvider;
    if (!provider) return;

    const branched = this.messages.slice(0, index + 1);
    try {
      const conversation = await aiConversationService.createConversation(
        this.selectedConversation?.title ?? 'New chat',
        provider,
        this.selectedConversation?.model ?? this.selectedModel
      );
      await aiConversationService.saveConversation(
        conversation.id,
        toWireMessages(conversation.id, branched)
      );

      this.applyConversationSelection(conversation);
      this.messages = branched;
      this.conversationId = conversation.id;
      await this.loadConversations();

      this.setUrlConversationId(conversation.id);
      toastActions.success('Branched into a new conversation');
    } catch (error) {
      toastActions.error('Failed to branch conversation', error);
    }
  };

  handleTruncate = async (index: number) => {
    if (this.isSending) return;
    this.messages = this.messages.slice(0, index);
    try {
      await this.persistMessages();
      await this.loadConversations();
    } catch (error) {
      toastActions.error('Failed to update conversation', error);
    }
  };

  handleRegenerate = async () => {
    if (this.isSending) return;
    const lastUserIndex = this.messages.map((m) => m.role).lastIndexOf('user');
    if (lastUserIndex < 0) return;

    const prompt = this.messages[lastUserIndex].content;
    const history = this.messages.slice(0, lastUserIndex);
    this.messages = history;
    await this.handleSendMessage(prompt, history);
  };

  /** Re-fetch the catalog; that round trip is the reachability check. */
  retryPlatform = async () => {
    await Promise.all([aiPlatform.refresh(), this.getSelector()?.reload()]);
  };

  handleStop = async () => {
    if (!this.activeStreamId) return;
    try {
      await aiChatService.cancelStream(this.activeStreamId);
    } catch (error) {
      toastActions.error('Failed to stop generation', error);
    }
  };
}
