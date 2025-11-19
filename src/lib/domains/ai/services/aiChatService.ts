import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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
   * Send a message to the AI provider (non-streaming)
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
    options: StreamMessageOptions = {}
  ): Promise<string> {
    // Generate unique stream ID
    const streamId = `stream-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
    
    // Set up event listeners before invoking the command
    const chunkEventName = `ai-stream-chunk-${streamId}`;
    const completeEventName = `ai-stream-complete-${streamId}`;
    
    let fullResponse = '';
    let isComplete = false;
    let streamError: Error | null = null;
    
    // Listen for chunks
    const chunkUnlisten = await listen<string>(chunkEventName, (event) => {
      const chunk = event.payload;
      fullResponse += chunk;
      if (options.onChunk) {
        options.onChunk(chunk);
      }
    });
    
    // Listen for completion
    const completeUnlisten = await listen<string>(completeEventName, (event) => {
      isComplete = true;
      const finalResponse = event.payload;
      if (options.onComplete) {
        options.onComplete(finalResponse);
      }
    });
    
    try {
      // Invoke the streaming command
      // Note: Tauri v2 expects camelCase parameter names from frontend
      const result = await invoke<string>('ai_send_message_stream', {
        message,
        history: history.map((msg) => ({
          role: msg.role,
          content: msg.content,
        })),
        provider: options.provider || null,
        conversationId: options.conversation_id || null,
        temperature: options.temperature || null,
        maxTokens: options.max_tokens || null,
        model: options.model || null,
        streamId: streamId,
      });
      
      // Wait for completion event (with timeout)
      let attempts = 0;
      const maxAttempts = 200; // 10 seconds max wait
      while (!isComplete && attempts < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, 50));
        attempts++;
      }
      
      if (!isComplete) {
        console.warn('Stream completion event not received, using accumulated response');
      }
      
      // Clean up listeners
      await chunkUnlisten();
      await completeUnlisten();
      
      if (streamError) {
        throw streamError;
      }
      
      return result || fullResponse;
    } catch (error) {
      // Clean up listeners on error
      await chunkUnlisten();
      await completeUnlisten();
      
      const err = error instanceof Error ? error : new Error(String(error));
      if (options.onError) {
        options.onError(err);
      }
      throw err;
    }
  }

  /**
   * Clear chat history (client-side only)
   */
  clearChat(): void {
    // This is a client-side operation, no backend call needed
  }
}

export const aiChatService = new AIChatService();

