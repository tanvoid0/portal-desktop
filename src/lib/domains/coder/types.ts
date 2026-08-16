// UI-only shapes for the coder domain.
//
// Everything that also exists in Rust is generated from it — see `./generated/`,
// produced by ts-rs from `portal_core::domains::coder::{types, diff}` and
// refreshed by `pnpm gen:types`. These types used to be hand-written mirrors,
// which is exactly the drift the generator removes; what is left below has no
// Rust counterpart (event payloads and the git-commit panel's own shapes).

export type {
  ChangeStatus,
  ChatMessage,
  CoderAgentMode,
  CoderRunResult,
  CoderSubAgent,
  CoderSubAgentStatus,
  CoderThreadKind,
  CoderThreadSummary,
  FileChange,
  GitHubIssueRef,
  Hunk,
  MultitaskCancelRequest,
  MultitaskCleanupRequest,
  MultitaskSpawnRequest,
  PendingApproval,
  PermissionMode,
  PermissionRule,
  SpawnSubAgentTask,
  ToolCall,
} from './generated';

import type { CoderThread as CoderThreadWire, CoderThreadSummary } from './generated';

/**
 * The wire thread plus the one field only the UI has.
 *
 * `message_count` is not on the Rust struct: a sidebar row is built from a
 * `CoderThreadSummary`, which carries the count because the row has no
 * `messages` to count. Declared as an extension rather than folded into the
 * generated type so every *other* field stays checked against Rust.
 */
export interface CoderThread extends CoderThreadWire {
  /** Present on rows built from a summary; a real thread counts `messages`. */
  message_count?: number;
}

export function summaryToThread(summary: CoderThreadSummary): CoderThread {
  return {
    id: summary.id,
    title: summary.title,
    workspace_root: summary.workspace_root,
    project_id: summary.project_id,
    model: summary.model,
    llm_provider: summary.llm_provider,
    platform_thread_id: summary.platform_thread_id,
    messages: [],
    thread_kind: summary.thread_kind,
    created_at: summary.created_at,
    updated_at: summary.updated_at,
    message_count: summary.message_count,
  };
}

export interface RunCommandEvent {
  thread_id: string;
  call_id: string;
  command: string;
  terminal_id?: string | null;
  workspace_root: string;
}

export interface ListTerminalsEvent {
  thread_id: string;
  call_id: string;
}

export interface ThreadTitleEvent {
  thread_id: string;
  title: string;
}

export interface CoderDoneEvent {
  thread_id: string;
  final_text: string | null;
  exhausted: boolean;
  cancelled?: boolean;
  title?: string;
}

export interface GitDiffStats {
  isRepo: boolean;
  branch?: string | null;
  additions: number;
  deletions: number;
  changedFiles: number;
  hasChanges: boolean;
}

export interface GitFileChange {
  path: string;
  status: string;
  additions: number;
  deletions: number;
  diff: string;
}

export interface GitCommitDraft {
  branch?: string | null;
  title: string;
  summary: string;
  changes: GitFileChange[];
  aiGenerated: boolean;
}
