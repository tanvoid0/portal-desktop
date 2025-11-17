import { invoke } from '@tauri-apps/api/core';
import type { Conversation, ConversationMessage, ProviderType } from '../types/index.js';

export class AIConversationService {
  /**
   * Create a new named conversation
   */
  async createConversation(title: string, provider: ProviderType): Promise<Conversation> {
    return invoke<Conversation>('ai_create_conversation', { title, provider });
  }

  /**
   * Save conversation messages
   */
  async saveConversation(id: string, messages: ConversationMessage[]): Promise<void> {
    return invoke('ai_save_conversation', { id, messages });
  }

  /**
   * Load conversation by ID
   */
  async loadConversation(id: string): Promise<{ conversation: Conversation; messages: ConversationMessage[] }> {
    return invoke<{ conversation: Conversation; messages: ConversationMessage[] }>('ai_load_conversation', { id });
  }

  /**
   * List all conversations with metadata
   */
  async listConversations(): Promise<Conversation[]> {
    return invoke<Conversation[]>('ai_list_conversations');
  }

  /**
   * Delete conversation
   */
  async deleteConversation(id: string): Promise<void> {
    return invoke('ai_delete_conversation', { id });
  }

  /**
   * Update conversation title
   */
  async updateConversationTitle(id: string, title: string): Promise<void> {
    return invoke('ai_update_conversation_title', { id, title });
  }
}

export const aiConversationService = new AIConversationService();

