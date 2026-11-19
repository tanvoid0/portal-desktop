// One place that maps every backend's tool name/args onto a `ToolKind` plus
// canonical args, so display, summaries and renderers never branch on raw names.
// Sources: Portal coder (snake_case), Claude Code native (Bash, Read, ...),
// Cursor CLI (`<kind>ToolCall` with the suffix stripped: shell, read, ...) and
// Codex (`commandExecution`).

export type ToolKind =
  | 'read'
  | 'write'
  | 'edit'
  | 'multi-edit'
  | 'command'
  | 'search'
  | 'list'
  | 'todo'
  | 'subagent'
  | 'web'
  | 'fallback';

export interface NormalizedTool {
  kind: ToolKind;
  args: Record<string, unknown>;
}

const KINDS: Record<string, ToolKind> = {
  // Portal
  read_file: 'read',
  write_file: 'write',
  edit_file: 'edit',
  list_dir: 'list',
  search_files: 'search',
  run_command: 'command',
  delegate_task: 'subagent',
  spawn_parallel_tasks: 'subagent',
  // Claude Code
  bash: 'command',
  read: 'read',
  write: 'write',
  edit: 'edit',
  multiedit: 'multi-edit',
  notebookedit: 'edit',
  grep: 'search',
  glob: 'search',
  ls: 'list',
  todowrite: 'todo',
  task: 'subagent',
  webfetch: 'web',
  websearch: 'web',
  // Cursor / Codex
  shell: 'command',
  commandexecution: 'command',
  todo: 'todo',
};

function pick(o: Record<string, unknown>, ...keys: string[]): unknown {
  for (const k of keys) if (o[k] !== undefined && o[k] !== null) return o[k];
  return undefined;
}

export function normalizeTool(name: string, rawArgs: unknown): NormalizedTool {
  const o = (rawArgs && typeof rawArgs === 'object' ? rawArgs : {}) as Record<string, unknown>;
  const kind = KINDS[name.toLowerCase()] ?? 'fallback';
  const path = pick(o, 'path', 'file_path', 'notebook_path', 'target_notebook');
  switch (kind) {
    case 'read':
    case 'list':
      return { kind, args: { path } };
    case 'write':
      return { kind, args: { path, content: pick(o, 'content', 'contents', 'fileText') } };
    case 'edit':
      return {
        kind,
        args: {
          path,
          old_string: pick(o, 'old_string', 'old_source'),
          new_string: pick(o, 'new_string', 'new_source'),
          replace_all: o.replace_all,
        },
      };
    case 'multi-edit':
      return { kind, args: { path, edits: Array.isArray(o.edits) ? o.edits : [] } };
    case 'command':
      // Cursor shell passes `command`, Codex too; Bash adds description/timeout we drop.
      return { kind, args: { command: pick(o, 'command', 'cmd') } };
    case 'search':
      return {
        kind,
        args: { query: pick(o, 'query', 'pattern', 'glob_pattern', 'q'), path },
      };
    case 'todo':
      return { kind, args: { todos: pick(o, 'todos', 'items') ?? [] } };
    case 'subagent':
      // Portal keeps its own keys (goal, tasks, issue_urls); Claude Task adds description/prompt.
      return {
        kind,
        args: { ...o, description: pick(o, 'description', 'goal'), prompt: o.prompt },
      };
    case 'web':
      return { kind, args: { url: o.url, query: o.query } };
    default:
      return { kind, args: o };
  }
}
