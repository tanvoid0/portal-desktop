import type { CoderSubAgent, ToolCall } from '../../types.js';
import type { ToolKind } from '../../utils/toolKinds.js';

/** Props every tool body renderer receives from `ToolCallCard`. */
export interface ToolBodyProps {
  call: ToolCall;
  kind: ToolKind;
  /** Canonical args from `normalizeTool`. */
  args: Record<string, unknown>;
  result: string | null;
  failed: boolean;
  /** Result with the noisy failure prefix stripped. */
  failedOutput: string;
  workspaceRoot: string;
  threadId?: string;
  subAgents: CoderSubAgent[];
  coordinatorId: string;
  onOpenSubAgent?: (childThreadId: string, coordinatorId: string) => void;
  onCancelSubAgent?: (subAgentId: string) => void;
  onCleanupSubAgent?: (subAgentId: string) => void;
}
