import { invoke } from '@tauri-apps/api/core';
import type { ChatMessage, ProviderType } from '../types/index.js';

export interface SendMessageOptions {
  provider?: ProviderType;
  conversation_id?: string;
  temperature?: number;
  max_tokens?: number;
  model?: string;
}

export interface StreamMessageOptions extends SendMessageOptions {
  onChunk?: (chunk: string) => void;
  onComplete?: (fullMessage: string) => void;
  onError?: (error: Error) => void;
}

export class AIChatService {
  /**
   * Send a message to the AI provider
   */
  async sendMessage(
    message: string,
    history: ChatMessage[] = [],
    options: SendMessageOptions = {}
  ): Promise<string> {
    return invoke<string>('ai_send_message', {
      message,
      history: history.map((msg) => ({
        role: msg.role,
        content: msg.content,
      })),
      provider: options.provider || null,
      conversation_id: options.conversation_id || null,
      temperature: options.temperature || null,
      max_tokens: options.max_tokens || null,
      model: options.model || null,
    });
  }

  /**
   * Stream a message from the AI provider
   */
  async streamMessage(
    message: string,
    history: ChatMessage[] = [],
    options: StreamMessageOptions = {}
  ): Promise<string> {
    // For now, we'll use a simple approach with invoke
    // In the future, this can be enhanced with SSE or WebSocket
    const fullResponse = await this.sendMessage(message, history, options);
    
    if (options.onChunk) {
      // Simulate streaming by chunking the response
      const words = fullResponse.split(' ');
      for (let i = 0; i < words.length; i++) {
        const chunk = (i === 0 ? '' : ' ') + words[i];
        options.onChunk(chunk);
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    }
    
    if (options.onComplete) {
      options.onComplete(fullResponse);
    }
    
    return fullResponse;
  }

  /**
   * Clear chat history (client-side only)
   */
  clearChat(): void {
    // This is a client-side operation, no backend call needed
  }
}

export const aiChatService = new AIChatService();

