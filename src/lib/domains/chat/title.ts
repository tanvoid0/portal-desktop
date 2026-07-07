export type ChatPlaceholderTitle = "New chat" | "New session";

export interface ThreadTitleEvent {
  thread_id: string;
  title: string;
}

export interface ChatDonePayload {
  thread_id: string;
  title: string;
  messages: Array<{ role: string; content?: string; [key: string]: unknown }>;
}

export function isPlaceholderTitle(title: string | null | undefined): boolean {
  if (!title?.trim()) return true;
  return title === "New chat" || title === "New session";
}

/** Client-side optimistic fallback (matches server). */
export function fallbackTitleFromMessage(
  message: string,
  defaultTitle: ChatPlaceholderTitle = "New chat",
): string {
  const text = message.trim().replace(/\s+/g, " ");
  if (!text) return defaultTitle;
  if (text.length <= 48) return text;
  return `${text.slice(0, 45)}...`;
}

/** Prefer smarter title over placeholder/fallback on reconcile. */
export function reconcileThreadTitle(
  current: string | null | undefined,
  incoming: string,
  fallback?: string,
): string {
  if (!incoming.trim()) return current?.trim() || incoming;
  if (isPlaceholderTitle(incoming) && !isPlaceholderTitle(current)) {
    return current!;
  }
  if (
    fallback &&
    incoming === fallback &&
    current &&
    !isPlaceholderTitle(current) &&
    current !== fallback
  ) {
    return current;
  }
  return incoming;
}
