import { normalizeTool } from './toolKinds.js';

export interface ToolCallDisplay {
  label: string;
  detail: string;
}

export interface CompactToolLine {
  /** Primary action verb, e.g. "Edited", "Read", "Ran". */
  verb: string;
  /** File path or command target. */
  target: string;
  additions?: number;
  deletions?: number;
}

export type ToolResultStatus = 'pending' | 'success' | 'failed';

function str(args: Record<string, unknown>, key: string, fallback = ''): string {
  const v = args[key];
  if (v === undefined || v === null || v === '') return fallback;
  return String(v);
}

/** Human-readable header for a coder tool bubble. */
export function getToolCallDisplay(
  tool: string,
  rawArgs: Record<string, unknown>
): ToolCallDisplay {
  const { kind, args } = normalizeTool(tool, rawArgs);
  switch (kind) {
    case 'read':
      return { label: 'Reading file', detail: str(args, 'path') };
    case 'write':
      return { label: 'Writing file', detail: str(args, 'path') };
    case 'edit':
    case 'multi-edit':
      return { label: 'Editing file', detail: str(args, 'path') };
    case 'list':
      return { label: 'Reading directory', detail: str(args, 'path', '.') };
    case 'search':
      return {
        label: 'Searching files',
        detail: `"${str(args, 'query')}" in ${str(args, 'path', '.')}`,
      };
    case 'command':
      return { label: 'Running command', detail: str(args, 'command') };
    case 'todo':
      return { label: 'Updating todos', detail: '' };
    case 'web':
      return { label: 'Fetching web', detail: str(args, 'url') || str(args, 'query') };
    case 'subagent': {
      if (tool !== 'spawn_parallel_tasks') {
        return { label: 'Delegating task', detail: str(args, 'description') };
      }
      const tasks = args.tasks;
      const count = Array.isArray(tasks) ? tasks.length : 0;
      const issues = args.issue_urls;
      const issueCount = Array.isArray(issues) ? issues.length : 0;
      const total = count || issueCount;
      return {
        label: 'Spawning parallel tasks',
        detail: total ? `${total} task${total === 1 ? '' : 's'}` : '',
      };
    }
    default:
      return {
        label: tool === 'list_terminals' ? 'Listing terminals' : tool.replace(/_/g, ' '),
        detail: '',
      };
  }
}

/** Whether a completed tool result represents a failure. */
export function isToolResultFailure(tool: string, result: string): boolean {
  const text = result.trim();
  if (!text) return false;

  if (text.startsWith('Error: ')) return true;

  const lower = text.toLowerCase();
  if (
    lower.startsWith('rejected') ||
    lower.startsWith('denied') ||
    lower.includes('blocked by deny rule') ||
    lower.startsWith('plan mode:')
  ) {
    return true;
  }

  if (normalizeTool(tool, {}).kind === 'command') {
    const match = text.match(/^exit (-?\d+)/);
    if (match) return Number.parseInt(match[1], 10) !== 0;
  }

  return false;
}

export function getToolResultStatus(
  tool: string,
  result: string | null | undefined
): ToolResultStatus {
  if (result == null) return 'pending';
  return isToolResultFailure(tool, result) ? 'failed' : 'success';
}

/** Short failure summary for the collapsed bubble header. */
export function getToolResultFailureSummary(tool: string, result: string): string {
  if (result.startsWith('Error: ')) {
    const msg = result.slice('Error: '.length).trim();
    const first = msg.split('\n')[0] ?? msg;
    return first.length > 80 ? `${first.slice(0, 77)}…` : first;
  }

  if (normalizeTool(tool, {}).kind === 'command') {
    const match = result.match(/^exit (-?\d+)/);
    if (match) return `Command failed (exit ${match[1]})`;
  }

  const first = result.trim().split('\n')[0] ?? result;
  return first.length > 80 ? `${first.slice(0, 77)}…` : first;
}

/** Body text to show for a failed result (strips noisy prefixes where helpful). */
export function formatFailedResult(tool: string, result: string): string {
  if (result.startsWith('Error: ')) {
    return result.slice('Error: '.length).trim();
  }

  if (normalizeTool(tool, {}).kind === 'command') {
    return result.replace(/^exit -?\d+\n?/, '').trim() || result.trim();
  }

  return result.trim();
}

/** Split tool output into display lines, trimming empties. */
export function resultLines(result: string): string[] {
  return result
    .split('\n')
    .map((l) => l.trimEnd())
    .filter((l) => l.length > 0);
}

/** Workspace label for command bubbles (`in <path>`). */
export function formatCommandCwd(workspaceRoot: string): string {
  const trimmed = workspaceRoot.trim();
  return trimmed ? `in ${trimmed}` : '';
}

function basename(path: string): string {
  const normalized = path.replace(/\\/g, '/');
  const parts = normalized.split('/').filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function countLines(text: string): number {
  if (!text) return 0;
  return text.split('\n').length;
}

/** Line-level diff stats for edit_file tool headers. */
export function countEditDiff(
  oldString: string,
  newString: string
): { additions: number; deletions: number } {
  return {
    deletions: countLines(oldString),
    additions: countLines(newString),
  };
}

/** Cursor-style one-line summary for a tool action. */
export function getCompactToolLine(
  tool: string,
  rawArgs: Record<string, unknown>
): CompactToolLine {
  const { kind, args } = normalizeTool(tool, rawArgs);
  switch (kind) {
    case 'read':
      return { verb: 'Read', target: basename(str(args, 'path')) };
    case 'write':
      return { verb: 'Wrote', target: basename(str(args, 'path')) };
    case 'edit': {
      const diff = countEditDiff(str(args, 'old_string'), str(args, 'new_string'));
      return {
        verb: 'Edited',
        target: basename(str(args, 'path')),
        additions: diff.additions,
        deletions: diff.deletions,
      };
    }
    case 'multi-edit': {
      const edits = (Array.isArray(args.edits) ? args.edits : []) as Array<Record<string, unknown>>;
      let additions = 0;
      let deletions = 0;
      for (const e of edits) {
        const d = countEditDiff(str(e, 'old_string'), str(e, 'new_string'));
        additions += d.additions;
        deletions += d.deletions;
      }
      return { verb: 'Edited', target: basename(str(args, 'path')), additions, deletions };
    }
    case 'list':
      return { verb: 'Listed', target: str(args, 'path', '.') };
    case 'search':
      return {
        verb: 'Searched',
        target: `"${str(args, 'query')}"`,
      };
    case 'command': {
      const cmd = str(args, 'command');
      const short = cmd.length > 48 ? `${cmd.slice(0, 45)}…` : cmd;
      return { verb: 'Ran', target: short };
    }
    case 'todo':
      return { verb: 'Updated todos', target: '' };
    case 'web':
      return { verb: 'Fetched', target: str(args, 'url') || `"${str(args, 'query')}"` };
    case 'subagent': {
      if (tool !== 'spawn_parallel_tasks') {
        return { verb: 'Delegated', target: str(args, 'description').slice(0, 48) };
      }
      const tasks = args.tasks;
      const count = Array.isArray(tasks) ? tasks.length : 0;
      return {
        verb: 'Spawned',
        target: `${count} parallel task${count === 1 ? '' : 's'}`,
      };
    }
    default:
      return { verb: tool.replace(/_/g, ' '), target: '' };
  }
}
