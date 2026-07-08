import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ChatMessage, ProviderType } from "../types/index.js";

export interface ConversationTitleEvent {
  conversation_id: string;
  title: string;
}

export interface StreamCompletePayload {
  content: string;
  title?: string;
}

export interface SendMessageOptions {
  provider?: ProviderType;
  /** agent-platform backend id (ollama, lm_studio, gemini, …) */
  llm_provider?: string;
  conversation_id?: string;
  temperature?: number;
  max_tokens?: number;
  model?: string;
}

export interface StreamMessageOptions extends SendMessageOptions {
  onChunk?: (chunk: string) => void;
  onComplete?: (fullMessage: string, payload?: StreamCompletePayload) => void;
  onTitleUpdated?: (event: ConversationTitleEvent) => void;
  onError?: (error: Error) => void;
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
      history: history.map((msg) => ({
        role: msg.role,
        content: msg.content,
      })),
      provider: options.provider || null,
      llmProvider: options.llm_provider || null,
      conversationId: options.conversation_id || null,
      temperature: options.temperature || null,
      maxTokens: options.max_tokens || null,
      model: options.model || null,
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
        history: history.map((msg) => ({
          role: msg.role,
          content: msg.content,
        })),
        provider: options.provider || null,
        llmProvider: options.llm_provider || null,
        conversationId: options.conversation_id || null,
        temperature: options.temperature || null,
        maxTokens: options.max_tokens || null,
        model: options.model || null,
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

  clearChat(): void {
    // client-side only
  }
}

export const aiChatService = new AIChatService();
