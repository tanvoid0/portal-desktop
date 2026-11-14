import { goto } from '$app/navigation';
import { agentRuns } from '../state/agentRuns.svelte.js';

export interface OpenAgentsSessionOptions {
  cwd: string;
  prompt?: string;
  taskId?: string;
}

/**
 * Hand off to the Agents workspace: stash cwd/prompt/task into the shared
 * `pendingLaunch` slot `AgentsWorkspace` consumes on mount, then navigate.
 * The shared entry point behind every "Run agent here" action — sibling to
 * coder's `openCoderSession`.
 */
export function openAgentsSession(opts: OpenAgentsSessionOptions): void {
  agentRuns.pendingLaunch = opts;
  void goto('/ai/agents');
}

/** Jumps straight to an existing session. */
export function openAgentsSessionById(id: string): void {
  void goto('/ai/agents?session=' + encodeURIComponent(id));
}
