import type { ChatMessage, ToolCall } from '../types.js';
import { getToolResultStatus } from './toolCallDisplay.js';
import { normalizeTool } from './toolKinds.js';

/** One phrase in a grouped activity summary, e.g. "Running" + " a command". */
export type ActivitySummaryPart = {
  verb: string;
  rest: string;
  /** Bold verb for in-progress operations. */
  isActive: boolean;
};

export type FeedBlock =
  | {
      kind: 'user';
      messageIndex: number;
      message: ChatMessage;
    }
  | {
      kind: 'thought';
      messageIndex: number;
      message: ChatMessage;
      durationMs: number | null;
    }
  | {
      kind: 'assistant';
      messageIndex: number;
      message: ChatMessage;
      responseLatencyMs: number | null;
    }
  | {
      kind: 'activity';
      /** Summary like "Ran a command, read a file". */
      parts: ActivitySummaryPart[];
      isRunning: boolean;
      hasFailed: boolean;
      tools: Array<{
        messageIndex: number;
        call: ToolCall;
        result: string | null;
      }>;
    }
  | {
      kind: 'tool';
      messageIndex: number;
      call: ToolCall;
      result: string | null;
    };

type ToolBucket = {
  total: number;
  pending: number;
  failed: number;
};

const ACTIVITY_ORDER = [
  'edit',
  'write',
  'read',
  'list',
  'search',
  'command',
  'todo',
  'web',
  'subagent',
  'spawn_parallel_tasks',
] as const;

function parseTimestamp(value?: string | null): number | null {
  if (!value) return null;
  const ts = new Date(value).getTime();
  return Number.isNaN(ts) ? null : ts;
}

function capitalize(text: string): string {
  if (!text) return text;
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** `key` is a ToolKind, or the raw tool name for kind 'fallback'. */
function activityPhrase(key: string, count: number, active: boolean): ActivitySummaryPart | null {
  const plural = count > 1;
  const files = plural ? ` ${count} files` : ' a file';
  const [gerund, past, rest] = ((): [string, string, string] => {
    switch (key) {
      case 'edit':
        return ['Editing', 'Edited', files];
      case 'write':
        return ['Creating', 'Created', files];
      case 'read':
        return ['Reading', 'Read', files];
      case 'list':
        return ['Listing', 'Listed', plural ? ` ${count} directories` : ' a directory'];
      case 'search':
        return ['Searching', plural ? `${count} searches` : 'Searched', plural ? '' : ' files'];
      case 'command':
        return ['Running', 'Ran', plural ? ` ${count} commands` : ' a command'];
      case 'todo':
        return ['Updating', 'Updated', ' todos'];
      case 'web':
        return ['Fetching', 'Fetched', plural ? ` ${count} pages` : ' a page'];
      case 'spawn_parallel_tasks':
        return ['Spawning', 'Spawned', ' parallel tasks'];
      case 'subagent':
        return ['Delegating', 'Delegated', plural ? ` ${count} tasks` : ' a task'];
      default:
        return ['Running', 'Ran', ` ${key.replace(/_/g, ' ')}`];
    }
  })();
  // 'Searching' keeps its in-progress rest ("N times") from before.
  if (active) {
    return {
      verb: gerund,
      rest: key === 'search' && plural ? ` ${count} times` : rest,
      isActive: true,
    };
  }
  return { verb: past, rest, isActive: false };
}

/** Build Cursor-style summary parts for a consecutive tool run. */
export function buildActivitySummary(tools: Array<{ call: ToolCall; result: string | null }>): {
  parts: ActivitySummaryPart[];
  isRunning: boolean;
  hasFailed: boolean;
} {
  const buckets = new Map<string, ToolBucket>();
  let isRunning = false;
  let hasFailed = false;

  for (const { call, result } of tools) {
    const tool = call.function.name;
    const { kind } = normalizeTool(tool, {});
    const key =
      kind === 'fallback' || tool === 'spawn_parallel_tasks'
        ? tool
        : kind === 'multi-edit'
          ? 'edit'
          : kind;
    const bucket = buckets.get(key) ?? { total: 0, pending: 0, failed: 0 };
    bucket.total++;
    const status = getToolResultStatus(tool, result);
    if (status === 'pending') {
      bucket.pending++;
      isRunning = true;
    } else if (status === 'failed') {
      bucket.failed++;
      hasFailed = true;
    }
    buckets.set(key, bucket);
  }

  const orderedTools = [
    ...ACTIVITY_ORDER.filter((tool) => buckets.has(tool)),
    ...[...buckets.keys()].filter(
      (tool) => !ACTIVITY_ORDER.includes(tool as (typeof ACTIVITY_ORDER)[number])
    ),
  ];

  const parts: ActivitySummaryPart[] = [];
  for (const tool of orderedTools) {
    const bucket = buckets.get(tool);
    if (!bucket) continue;
    const active = bucket.pending > 0;
    // A failed call did no work: never let it inflate the count, or the summary
    // claims work that never happened (e.g. "Created 2 files" for two
    // `Error: missing arg: path` results).
    const succeeded = bucket.total - bucket.failed;

    if (succeeded > 0) {
      const phrase = activityPhrase(tool, succeeded, active);
      if (phrase) {
        parts.push(
          bucket.failed > 0
            ? { ...phrase, rest: `${phrase.rest} (${bucket.failed} failed)` }
            : phrase
        );
      }
      continue;
    }

    if (bucket.failed > 0) {
      // Nothing succeeded. Reuse the in-progress phrasing ("Creating a file")
      // as a gerund so no extra per-tool vocabulary is needed.
      const gerund = activityPhrase(tool, bucket.failed, true);
      if (gerund) {
        parts.push({
          verb: 'Failed',
          rest: ` ${gerund.verb.toLowerCase()}${gerund.rest}`,
          isActive: false,
        });
      }
    }
  }

  if (parts.length > 0) {
    const first = parts[0];
    parts[0] = { ...first, verb: capitalize(first.verb) };
  }

  return { parts, isRunning, hasFailed };
}

function groupConsecutiveTools(blocks: FeedBlock[]): FeedBlock[] {
  const result: FeedBlock[] = [];
  let i = 0;

  while (i < blocks.length) {
    const block = blocks[i];
    if (block.kind !== 'tool') {
      result.push(block);
      i++;
      continue;
    }

    const run: Extract<FeedBlock, { kind: 'tool' }>[] = [];
    while (i < blocks.length && blocks[i].kind === 'tool') {
      run.push(blocks[i] as Extract<FeedBlock, { kind: 'tool' }>);
      i++;
    }

    if (run.length >= 2) {
      const tools = run.map(({ messageIndex, call, result: toolResult }) => ({
        messageIndex,
        call,
        result: toolResult,
      }));
      const { parts, isRunning, hasFailed } = buildActivitySummary(tools);
      if (parts.length > 0) {
        result.push({
          kind: 'activity',
          parts,
          isRunning,
          hasFailed,
          tools,
        });
        continue;
      }
    } else if (run.length === 1) {
      const tool = run[0];
      const status = getToolResultStatus(tool.call.function.name, tool.result);
      if (status === 'pending') {
        const tools = [
          {
            messageIndex: tool.messageIndex,
            call: tool.call,
            result: tool.result,
          },
        ];
        const { parts, isRunning, hasFailed } = buildActivitySummary(tools);
        if (parts.length > 0) {
          result.push({
            kind: 'activity',
            parts,
            isRunning,
            hasFailed,
            tools,
          });
          continue;
        }
      }
    }

    result.push(...run);
  }

  return result;
}

/** Flatten a transcript into Cursor-style chronological blocks. */
export function buildFeedBlocks(
  messages: ChatMessage[],
  resultsById: Map<string, string>
): FeedBlock[] {
  const blocks: FeedBlock[] = [];
  let lastUserTimestamp: number | null = null;

  for (let i = 0; i < messages.length; i++) {
    const message = messages[i];
    if (message.role === 'system' || message.role === 'tool') continue;

    if (message.role === 'user') {
      const timestamp = parseTimestamp(message.timestamp);
      if (timestamp != null) lastUserTimestamp = timestamp;
      blocks.push({ kind: 'user', messageIndex: i, message });
      continue;
    }

    if (message.role === 'assistant') {
      const timestamp = parseTimestamp(message.timestamp);
      const responseLatencyMs =
        timestamp != null && lastUserTimestamp != null
          ? Math.max(0, timestamp - lastUserTimestamp)
          : null;

      const hasTools = (message.tool_calls?.length ?? 0) > 0;
      const hasContent = !!message.content?.trim();

      if (hasContent && hasTools) {
        blocks.push({
          kind: 'thought',
          messageIndex: i,
          message,
          durationMs: responseLatencyMs,
        });
      } else if (hasContent) {
        blocks.push({
          kind: 'assistant',
          messageIndex: i,
          message,
          responseLatencyMs,
        });
      }

      for (const call of message.tool_calls ?? []) {
        blocks.push({
          kind: 'tool',
          messageIndex: i,
          call,
          result: resultsById.get(call.id) ?? null,
        });
      }

      if (!hasContent && !hasTools) {
        blocks.push({
          kind: 'assistant',
          messageIndex: i,
          message,
          responseLatencyMs,
        });
      }
    }
  }

  return groupConsecutiveTools(blocks);
}

export function formatWorkedDuration(ms: number | null): string {
  if (ms == null || ms < 0) return '';
  if (ms < 1000) return 'Worked for < 1s';
  return `Worked for ${formatElapsedSeconds(ms / 1000)}`;
}

/** 344 -> "5m 44s", 3720 -> "1h 2m", 42 -> "42s". */
export function formatElapsedSeconds(secs: number): string {
  const total = Math.round(secs);
  if (total < 60) return `${total}s`;
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return m === 0 ? `${h}h` : `${h}h ${m}m`;
  return s === 0 ? `${m}m` : `${m}m ${s}s`;
}

/** Same wording as the chat page's inline reasoning block. */
export { formatThoughtLabel as formatThoughtDuration } from '$lib/domains/ai/utils/thinking.js';
