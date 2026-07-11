/** Pseudo tool-call markup some models emit as plain text. */
const LEAKED_TOOL_RE = /<function=(\w+)[^>]*>(?:[\s\S]*?<\/function>)?/gi;

/** Remove leaked `<function=…>` markup from assistant text. */
export function stripLeakedToolSyntax(text: string): string {
  return text.replace(LEAKED_TOOL_RE, "").trim();
}

/** First tool name leaked as text, if any. */
export function findLeakedToolCall(text: string): string | null {
  LEAKED_TOOL_RE.lastIndex = 0;
  const match = LEAKED_TOOL_RE.exec(text);
  return match?.[1] ?? null;
}

export const LEAKED_TOOL_ERROR =
  "This model wrote a tool call as plain text instead of running it. Choose a model with tool/function calling support in AI → Providers, then retry.";
