import type { ChatMessage } from "../types.js";

function sameContent(a: ChatMessage, b: ChatMessage): boolean {
  return (
    (a.content ?? "") === (b.content ?? "") &&
    (a.tool_call_id ?? "") === (b.tool_call_id ?? "") &&
    JSON.stringify(a.tool_calls ?? null) === JSON.stringify(b.tool_calls ?? null)
  );
}

/** Preserve known timestamps and estimate missing ones for display. */
export function withMessageTimestamps(
  messages: ChatMessage[],
  previous: ChatMessage[] = [],
  anchorIso?: string,
): ChatMessage[] {
  const anchorMs = anchorIso ? Date.parse(anchorIso) : Date.now();
  const gapMs = 1500;

  return messages.map((message, index) => {
    if (message.timestamp) return message;

    const prev = previous[index];
    if (prev && prev.role === message.role && sameContent(prev, message) && prev.timestamp) {
      return { ...message, timestamp: prev.timestamp };
    }

    const prevMatch = previous.find(
      (p) => p.role === message.role && sameContent(p, message) && p.timestamp,
    );
    if (prevMatch?.timestamp) {
      return { ...message, timestamp: prevMatch.timestamp };
    }

    const estimatedMs = anchorMs - (messages.length - 1 - index) * gapMs;
    return { ...message, timestamp: new Date(estimatedMs).toISOString() };
  });
}
