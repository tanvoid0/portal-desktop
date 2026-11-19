import type { CoderThread } from '$lib/domains/coder/types.js';
import type { AgentSessionSummary } from '../generated';

/** An agent run shaped as a coder thread, so the agents sidebar can reuse CoderSessionList. */
/** `projectId` is the project the cwd resolves to (see groupSessions), so grouping survives subfolders. */
export function sessionToThread(
  session: AgentSessionSummary,
  projectId?: string | null
): CoderThread {
  return {
    id: session.id,
    title: session.title || session.backend_id,
    workspace_root: session.cwd,
    project_id: projectId != null ? Number.parseInt(projectId, 10) : null,
    model: session.model,
    llm_provider: session.backend_id,
    messages: [],
    thread_kind: 'session',
    created_at: session.started_at,
    updated_at: session.ended_at ?? session.started_at,
  };
}
