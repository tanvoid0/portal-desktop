// Projects the raw persisted event log onto rows the Timeline tab can render
// directly — mirrors `eventsToMessages.ts`'s "pure reducer" shape, but keeps
// every event instead of folding into chat turns. `TextDelta`/`ThinkingDelta`
// are never persisted (see `service.rs`'s pump loop — the `Message` event
// that follows already carries the finished text), so there is no chunk
// replay to fold here; the cases below are only a safety net for old rows.

import type { AgentEventRow } from '../generated';

export interface TimelineItem {
  id: string;
  created_at: string;
  label: string;
  detail?: string | null;
  isError?: boolean;
}

function truncate(value: string, max = 300): string {
  return value.length > max ? `${value.slice(0, max)}…` : value;
}

function stringifyInput(input: unknown): string {
  try {
    return truncate(JSON.stringify(input));
  } catch {
    return String(input);
  }
}

/** Pure fold over the persisted event log, in order. */
export function foldTimelineEvents(rows: AgentEventRow[]): TimelineItem[] {
  const items: TimelineItem[] = [];

  for (const row of rows) {
    const e = row.event;
    switch (e.type) {
      // No longer persisted — kept so an old row (persisted before this
      // change) still renders something instead of vanishing.
      case 'text_delta':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: 'Assistant text',
          detail: truncate(e.text),
        });
        break;
      case 'thinking_delta':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: 'Thinking',
          detail: truncate(e.text),
        });
        break;
      case 'session_started':
        items.push({ id: row.id, created_at: row.created_at, label: 'Session started' });
        break;
      case 'message':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: `${e.role} message`,
          detail: truncate(e.text),
        });
        break;
      case 'tool_call':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: `Tool call: ${e.name}`,
          detail: stringifyInput(e.input),
        });
        break;
      case 'tool_result':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: e.is_error ? 'Tool result (error)' : 'Tool result',
          detail: truncate(e.content),
          isError: e.is_error,
        });
        break;
      case 'subagent_started':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: `Subagent started${e.name ? `: ${e.name}` : ''}`,
        });
        break;
      case 'subagent_finished':
        items.push({ id: row.id, created_at: row.created_at, label: 'Subagent finished' });
        break;
      case 'permission_request':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: `Permission requested: ${e.tool_name}`,
          detail: stringifyInput(e.tool_input),
        });
        break;
      case 'permission_resolved':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: e.allowed ? 'Permission approved' : 'Permission denied',
          isError: !e.allowed,
        });
        break;
      case 'todos':
        items.push({ id: row.id, created_at: row.created_at, label: 'Todo list updated' });
        break;
      case 'usage':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: 'Usage',
          detail:
            e.total_cost_usd != null ? `Total cost: $${e.total_cost_usd.toFixed(4)}` : undefined,
        });
        break;
      case 'turn_done':
        items.push({ id: row.id, created_at: row.created_at, label: 'Turn finished' });
        break;
      case 'exited':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: e.code != null ? `Exited (code ${e.code})` : 'Exited',
          isError: !!e.code,
        });
        break;
      case 'raw':
        items.push({
          id: row.id,
          created_at: row.created_at,
          label: 'Raw',
          detail: truncate(e.line),
        });
        break;
      default: {
        // Exhaustiveness check: a new `AgentEvent` variant that isn't
        // handled above fails `pnpm check` here instead of silently
        // vanishing from the Timeline.
        const _exhaustive: never = e;
        break;
      }
    }
  }
  return items;
}
