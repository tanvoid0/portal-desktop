import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ChatMessage,
  ContextUsage,
  LlmUsage,
  ProviderType,
} from "../types/index.js";

export interface ConversationTitleEvent {
  conversation_id: string;
  title: string;
}

export interface StreamCompletePayload {
  content: string;
  title?: string;
  /** True when the user stopped generation — content is partial. */
  cancelled?: boolean;
  /** Present only when agent-platform reports usage on the stream. */
  context_usage?: ContextUsage | null;
  llm_usage?: LlmUsage | null;
  /** Upstream `choices[0].finish_reason`, when the provider reports one. */
  finish_reason?: string | null;
}

export interface SendMessageOptions {
  provider?: ProviderType;
  /** agent-platform backend id (ollama, lm_studio, gemini, …) */
  llm_provider?: string;
  conversation_id?: string;
  temperature?: number;
  max_tokens?: number;
  model?: string;
  /** Prepended to the history as a `system` turn; not stored in the thread. */
  system_prompt?: string;
  /** Extra sampling fields merged into the OpenAI-compatible request body. */
  extra_options?: Record<string, unknown>;
}

export interface StreamMessageOptions extends SendMessageOptions {
  onChunk?: (chunk: string) => void;
  onComplete?: (fullMessage: string, payload?: StreamCompletePayload) => void;
  onTitleUpdated?: (event: ConversationTitleEvent) => void;
  onError?: (error: Error) => void;
  /** Receives the stream id so the caller can pass it to `cancelStream`. */
  onStreamId?: (streamId: string) => void;
}

/** Backend `ChatMessage` takes any role string, so `system` passes through. */
function toWireHistory(history: ChatMessage[], systemPrompt?: string) {
  const wire = history.map((msg) => ({
    role: msg.role as string,
    content: msg.content,
  }));
  const trimmed = systemPrompt?.trim();
  if (trimmed) wire.unshift({ role: "system", content: trimmed });
  return wire;
}

export class AIChatService {
  /**
   * Send a message to the AI provider (non-streaming)
   */
  async sendMessage(
    message: string,
    history: ChatMessage[] = [],
    options: SendMessageOptions = {},
  ): Promise<string> {
    return invoke<string>("ai_send_message", {
      message,
      history: toWireHistory(history, options.system_prompt),
      provider: options.provider || null,
      llmProvider: options.llm_provider || null,
      conversationId: options.conversation_id || null,
      temperature: options.temperature || null,
      maxTokens: options.max_tokens || null,
      model: options.model || null,
      extraOptions: options.extra_options || null,
    });
  }

  /**
   * Stream a message from the AI provider using Tauri events
   */
  async streamMessage(
    message: string,
    history: ChatMessage[] = [],
    options: StreamMessageOptions = {},
  ): Promise<string> {
    const streamId = `stream-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
    options.onStreamId?.(streamId);

    const chunkEventName = `ai-stream-chunk-${streamId}`;
    const completeEventName = `ai-stream-complete-${streamId}`;
    const titleEventName = `ai-stream-title-${streamId}`;

    let fullResponse = "";
    let isComplete = false;
    let completePayload: StreamCompletePayload | undefined;
    let streamError: Error | null = null;

    const chunkUnlisten = await listen<string>(chunkEventName, (event) => {
      const chunk = event.payload;
      fullResponse += chunk;
      options.onChunk?.(chunk);
    });

    const titleUnlisten = await listen<ConversationTitleEvent>(
      titleEventName,
      (event) => {
        options.onTitleUpdated?.(event.payload);
      },
    );

    const completeUnlisten = await listen<StreamCompletePayload | string>(
      completeEventName,
      (event) => {
        isComplete = true;
        const payload = event.payload;
        if (typeof payload === "string") {
          completePayload = { content: payload };
          options.onComplete?.(payload);
        } else {
          completePayload = payload;
          options.onComplete?.(payload.content, payload);
        }
      },
    );

    try {
      const result = await invoke<string>("ai_send_message_stream", {
        message,
        history: toWireHistory(history, options.system_prompt),
        provider: options.provider || null,
        llmProvider: options.llm_provider || null,
        conversationId: options.conversation_id || null,
        temperature: options.temperature || null,
        maxTokens: options.max_tokens || null,
        model: options.model || null,
        extraOptions: options.extra_options || null,
        streamId: streamId,
      });

      let attempts = 0;
      const maxAttempts = 200;
      while (!isComplete && attempts < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, 50));
        attempts++;
      }

      if (!isComplete) {
        console.warn(
          "Stream completion event not received, using accumulated response",
        );
      }

      await chunkUnlisten();
      await titleUnlisten();
      await completeUnlisten();

      if (streamError) {
        throw streamError;
      }

      return completePayload?.content || result || fullResponse;
    } catch (error) {
      await chunkUnlisten();
      await titleUnlisten();
      await completeUnlisten();

      const err = error instanceof Error ? error : new Error(String(error));
      options.onError?.(err);
      throw err;
    }
  }

  /** Stop an in-flight stream; the backend completes it with partial content. */
  async cancelStream(streamId: string): Promise<void> {
    await invoke("ai_cancel_stream", { streamId });
  }

  clearChat(): void {
    // client-side only
  }
}

export const aiChatService = new AIChatService();
