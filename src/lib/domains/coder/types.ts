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
  timestamp?: string | null;
}

export interface CoderThread {
  id: string;
  title: string;
  workspace_root: string;
  model?: string | null;
  llm_provider?: string | null;
  platform_thread_id?: number | null;
  messages: ChatMessage[];
  thread_kind?: CoderThreadKind;
  created_at: string;
  updated_at: string;
  /** Present on sidebar rows loaded from summaries. */
  message_count?: number;
}

export interface CoderThreadSummary {
  id: string;
  title: string;
  workspace_root: string;
  model?: string | null;
  llm_provider?: string | null;
  platform_thread_id?: number | null;
  created_at: string;
  updated_at: string;
  thread_kind?: CoderThreadKind;
  message_count: number;
  is_running: boolean;
}

export function summaryToThread(summary: CoderThreadSummary): CoderThread {
  return {
    id: summary.id,
    title: summary.title,
    workspace_root: summary.workspace_root,
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

export type CoderThreadKind = "session" | "coordinator" | "sub-agent";

export interface PendingApproval {
  call_id: string;
  tool: string;
  arguments: Record<string, unknown>;
  suggested_rule: string;
  summary: string;
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

export type CoderSubAgentStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface GitHubIssueRef {
  owner: string;
  repo: string;
  number: number;
  url?: string | null;
}

export interface SpawnSubAgentTask {
  title: string;
  prompt: string;
  github_issue?: GitHubIssueRef | null;
  github_issue_url?: string | null;
}

export interface MultitaskSpawnRequest {
  coordinatorThreadId: string;
  baseRef?: string | null;
  tasks: SpawnSubAgentTask[];
  issueUrls?: string[] | null;
}

export interface MultitaskCancelRequest {
  coordinatorThreadId: string;
  subAgentId?: string | null;
}

export interface MultitaskCleanupRequest {
  coordinatorThreadId: string;
  subAgentIds: string[];
  force?: boolean;
}

export interface CoderSubAgent {
  id: string;
  coordinator_thread_id: string;
  child_thread_id: string;
  title: string;
  workspace_root: string;
  branch: string;
  status: CoderSubAgentStatus;
  github_owner?: string | null;
  github_repo?: string | null;
  github_issue_number?: number | null;
  github_issue_url?: string | null;
  result_summary?: string | null;
  error?: string | null;
  created_at: string;
  updated_at: string;
}
