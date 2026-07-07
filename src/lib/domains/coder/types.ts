// Types mirror the Rust `domains::coder::types` wire shapes.

export interface ToolCall {
  id: string;
  type: string;
  function: { name: string; arguments: string };
}

export interface ChatMessage {
  role: "system" | "user" | "assistant" | "tool";
  content?: string | null;
  tool_calls?: ToolCall[] | null;
  tool_call_id?: string | null;
}

export interface CoderThread {
  id: string;
  title: string;
  workspace_root: string;
  model?: string | null;
  platform_thread_id?: number | null;
  messages: ChatMessage[];
  created_at: string;
  updated_at: string;
}

export interface PendingApproval {
  call_id: string;
  tool: string;
  arguments: Record<string, unknown>;
  suggested_rule: string;
  summary: string;
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

export interface CoderRunResult {
  thread_id: string;
  messages: ChatMessage[];
  pending: PendingApproval | null;
  final_text: string | null;
  exhausted: boolean;
}

export interface Hunk {
  index: number;
  before_start: number;
  before_lines: string[];
  after_lines: string[];
  accepted: boolean;
}

export type ChangeStatus = "pending" | "accepted" | "rejected";

export interface FileChange {
  id: string;
  thread_id: string;
  path: string;
  tool: string;
  before: string;
  original_after: string;
  hunks: Hunk[];
  created: boolean;
  status: ChangeStatus;
  created_at: string;
}

export type PermissionMode = "auto-accept-all" | "review" | "plan";

export interface PermissionRule {
  tool: string;
  pattern: string;
  allow: boolean;
}

export interface GitDiffStats {
  isRepo: boolean;
  branch?: string | null;
  additions: number;
  deletions: number;
  changedFiles: number;
  hasChanges: boolean;
}
