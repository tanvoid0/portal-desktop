import type { ContextUsage } from "../types/index.js";

/** Human-readable labels for agent-platform context category keys. */
export const CONTEXT_CATEGORY_LABELS: Record<string, string> = {
  system_prompt: "System prompt",
  tools: "Tools",
  rules: "Rules",
  skills: "Skills",
  mcp: "MCP",
  subagents: "Subagents",
  conversation: "Conversation",
  injected_context: "Injected context",
};

export function formatTokenCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 10_000) return `${Math.round(n / 1000)}k`;
  if (n >= 1_000) return `${(n / 1000).toFixed(1)}k`;
  return n.toLocaleString();
}

export function contextBarColor(percentUsed: number): string {
  if (percentUsed >= 90) return "bg-destructive";
  if (percentUsed >= 75) return "bg-amber-500";
  return "bg-primary";
}

export function contextCategoriesForDisplay(
  usage: ContextUsage,
): Array<{ key: string; label: string; tokens: number }> {
  return Object.entries(usage.categories ?? {})
    .filter(([, tokens]) => tokens > 0)
    .map(([key, tokens]) => ({
      key,
      label: CONTEXT_CATEGORY_LABELS[key] ?? key.replace(/_/g, " "),
      tokens,
    }))
    .sort((a, b) => b.tokens - a.tokens);
}
