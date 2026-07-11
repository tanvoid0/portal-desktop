import type { ChatMessage, ToolCall } from "../types.js";
import { getToolResultStatus } from "./toolCallDisplay.js";

/** One phrase in a grouped activity summary, e.g. "Running" + " a command". */
export type ActivitySummaryPart = {
  verb: string;
  rest: string;
  /** Bold verb for in-progress operations. */
  isActive: boolean;
};

export type FeedBlock =
  | {
      kind: "user";
      messageIndex: number;
      message: ChatMessage;
    }
  | {
      kind: "thought";
      messageIndex: number;
      message: ChatMessage;
      durationMs: number | null;
    }
  | {
      kind: "assistant";
      messageIndex: number;
      message: ChatMessage;
      responseLatencyMs: number | null;
    }
  | {
      kind: "activity";
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
      kind: "tool";
      messageIndex: number;
      call: ToolCall;
      result: string | null;
    };

type ToolBucket = {
  total: number;
  pending: number;
};

const ACTIVITY_ORDER = [
  "edit_file",
  "write_file",
  "read_file",
  "list_dir",
  "search_files",
  "run_command",
  "delegate_task",
  "spawn_parallel_tasks",
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

function activityPhrase(
  tool: string,
  count: number,
  active: boolean,
): ActivitySummaryPart | null {
  const plural = count > 1;
  switch (tool) {
    case "edit_file":
      return active
        ? { verb: "Editing", rest: plural ? ` ${count} files` : " a file", isActive: true }
        : { verb: "Edited", rest: plural ? ` ${count} files` : " a file", isActive: false };
    case "write_file":
      return active
        ? { verb: "Creating", rest: plural ? ` ${count} files` : " a file", isActive: true }
        : { verb: "Created", rest: plural ? ` ${count} files` : " a file", isActive: false };
    case "read_file":
      return active
        ? { verb: "Reading", rest: plural ? ` ${count} files` : " a file", isActive: true }
        : { verb: "Read", rest: plural ? ` ${count} files` : " a file", isActive: false };
    case "list_dir":
      return active
        ? { verb: "Listing", rest: plural ? ` ${count} directories` : " a directory", isActive: true }
        : { verb: "Listed", rest: plural ? ` ${count} directories` : " a directory", isActive: false };
    case "search_files":
      if (active) {
        return {
          verb: "Searching",
          rest: plural ? ` ${count} times` : " files",
          isActive: true,
        };
      }
      return {
        verb: plural ? `${count} searches` : "Searched",
        rest: plural ? "" : " files",
        isActive: false,
      };
    case "run_command":
      return active
        ? { verb: "Running", rest: plural ? ` ${count} commands` : " a command", isActive: true }
        : { verb: "Ran", rest: plural ? ` ${count} commands` : " a command", isActive: false };
    case "delegate_task":
      return active
        ? { verb: "Delegating", rest: plural ? ` ${count} tasks` : " a task", isActive: true }
        : { verb: "Delegated", rest: plural ? ` ${count} tasks` : " a task", isActive: false };
    case "spawn_parallel_tasks":
      return active
        ? { verb: "Spawning", rest: " parallel tasks", isActive: true }
        : { verb: "Spawned", rest: " parallel tasks", isActive: false };
    default:
      return active
        ? {
            verb: "Running",
            rest: ` ${tool.replace(/_/g, " ")}`,
            isActive: true,
          }
        : {
            verb: "Ran",
            rest: ` ${tool.replace(/_/g, " ")}`,
            isActive: false,
          };
  }
}

/** Build Cursor-style summary parts for a consecutive tool run. */
export function buildActivitySummary(
  tools: Array<{ call: ToolCall; result: string | null }>,
): { parts: ActivitySummaryPart[]; isRunning: boolean; hasFailed: boolean } {
  const buckets = new Map<string, ToolBucket>();
  let isRunning = false;
  let hasFailed = false;

  for (const { call, result } of tools) {
    const tool = call.function.name;
    const bucket = buckets.get(tool) ?? { total: 0, pending: 0 };
    bucket.total++;
    const status = getToolResultStatus(tool, result);
    if (status === "pending") {
      bucket.pending++;
      isRunning = true;
    } else if (status === "failed") {
      hasFailed = true;
    }
    buckets.set(tool, bucket);
  }

  const orderedTools = [
    ...ACTIVITY_ORDER.filter((tool) => buckets.has(tool)),
    ...[...buckets.keys()].filter(
      (tool) => !ACTIVITY_ORDER.includes(tool as (typeof ACTIVITY_ORDER)[number]),
    ),
  ];

  const parts: ActivitySummaryPart[] = [];
  for (const tool of orderedTools) {
    const bucket = buckets.get(tool);
    if (!bucket) continue;
    const active = bucket.pending > 0;
    const count = bucket.total;
    const phrase = activityPhrase(tool, count, active);
    if (phrase) parts.push(phrase);
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
    if (block.kind !== "tool") {
      result.push(block);
      i++;
      continue;
    }

    const run: Extract<FeedBlock, { kind: "tool" }>[] = [];
    while (i < blocks.length && blocks[i].kind === "tool") {
      run.push(blocks[i] as Extract<FeedBlock, { kind: "tool" }>);
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
          kind: "activity",
          parts,
          isRunning,
          hasFailed,
          tools,
        });
        continue;
      }
    } else if (run.length === 1) {
      const tool = run[0];
      const status = getToolResultStatus(
        tool.call.function.name,
        tool.result,
      );
      if (status === "pending") {
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
            kind: "activity",
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
  resultsById: Map<string, string>,
): FeedBlock[] {
  const blocks: FeedBlock[] = [];
  let lastUserTimestamp: number | null = null;

  for (let i = 0; i < messages.length; i++) {
    const message = messages[i];
    if (message.role === "system" || message.role === "tool") continue;

    if (message.role === "user") {
      const timestamp = parseTimestamp(message.timestamp);
      if (timestamp != null) lastUserTimestamp = timestamp;
      blocks.push({ kind: "user", messageIndex: i, message });
      continue;
    }

    if (message.role === "assistant") {
      const timestamp = parseTimestamp(message.timestamp);
      const responseLatencyMs =
        timestamp != null && lastUserTimestamp != null
          ? Math.max(0, timestamp - lastUserTimestamp)
          : null;

      const hasTools = (message.tool_calls?.length ?? 0) > 0;
      const hasContent = !!message.content?.trim();

      if (hasContent && hasTools) {
        blocks.push({
          kind: "thought",
          messageIndex: i,
          message,
          durationMs: responseLatencyMs,
        });
      } else if (hasContent) {
        blocks.push({
          kind: "assistant",
          messageIndex: i,
          message,
          responseLatencyMs,
        });
      }

      for (const call of message.tool_calls ?? []) {
        blocks.push({
          kind: "tool",
          messageIndex: i,
          call,
          result: resultsById.get(call.id) ?? null,
        });
      }

      if (!hasContent && !hasTools) {
        blocks.push({
          kind: "assistant",
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
  if (ms == null || ms < 0) return "";
  if (ms < 1000) return "Worked for < 1s";
  const totalSeconds = Math.round(ms / 1000);
  if (totalSeconds < 60) return `Worked for ${totalSeconds}s`;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return seconds === 0
    ? `Worked for ${minutes}m`
    : `Worked for ${minutes}m ${seconds}s`;
}

export function formatThoughtDuration(ms: number | null): string {
  if (ms == null || ms < 0) return "Thought briefly";
  if (ms < 1000) return "Thought briefly";
  const totalSeconds = Math.round(ms / 1000);
  if (totalSeconds < 60) return `Thought for ${totalSeconds}s`;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return seconds === 0
    ? `Thought for ${minutes}m`
    : `Thought for ${minutes}m ${seconds}s`;
}
