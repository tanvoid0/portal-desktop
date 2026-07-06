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

export interface CoderRunResult {
  thread_id: string;
  messages: ChatMessage[];
  pending: PendingApproval | null;
  final_text: string | null;
  exhausted: boolean;
}

export type PermissionMode = "auto-accept-all" | "review" | "plan";

export interface PermissionRule {
  tool: string;
  pattern: string;
  allow: boolean;
}
