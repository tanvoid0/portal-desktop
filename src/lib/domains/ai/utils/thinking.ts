export interface ThinkingSplit {
  /** Scratchpad text, or null when the turn has no think block. */
  reasoning: string | null;
  answer: string;
  /** False while the closing tag has not streamed in yet. */
  closed: boolean;
}

/**
 * Reasoning models wrap their scratchpad in a leading `<think>` block — either
 * inline in the text, or folded in by the Rust provider from the upstream
 * `reasoning_content` delta.
 */
export function splitThinking(content: string): ThinkingSplit {
  const match = content.match(/^\s*<think(?:ing)?>([\s\S]*?)(?:<\/think(?:ing)?>|$)/);
  if (!match) return { reasoning: null, answer: content, closed: true };
  return {
    reasoning: match[1].trim(),
    answer: content.slice(match[0].length).trimStart(),
    closed: /<\/think(?:ing)?>/.test(match[0]),
  };
}

/** Summary line for a collapsed reasoning block, shared by chat and coder. */
export function formatThoughtLabel(ms: number | null | undefined): string {
  if (ms == null || ms < 1000) return 'Thought briefly';
  const totalSeconds = Math.round(ms / 1000);
  if (totalSeconds < 60) return `Thought for ${totalSeconds}s`;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return seconds === 0 ? `Thought for ${minutes}m` : `Thought for ${minutes}m ${seconds}s`;
}
