import type { ChatMessage, ConversationMessage } from '../types/index.js';

/** Stored conversation messages -> the chat panel's message shape. */
export function fromConversationMessages(list: ConversationMessage[]): ChatMessage[] {
  return list.map((msg) => ({
    role: msg.role,
    content: msg.content,
    timestamp: new Date(msg.timestamp),
  }));
}

/** Chat panel messages -> the wire shape persisted by `saveConversation`. */
export function toWireMessages(id: string, list: ChatMessage[]) {
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
