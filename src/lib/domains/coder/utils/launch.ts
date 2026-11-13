import { goto } from '$app/navigation';
import { coderSession } from '../state/coderSession.svelte.js';
import { coderUi } from '../state/coderUi.svelte';

export interface OpenCoderSessionOptions {
  /** Folder the agent works in. When set, it becomes the active workspace. */
  workspaceRoot?: string;
  /** Portal project id for that workspace, if known. */
  projectId?: string | null;
  /** Pre-fill the composer with this. NOT auto-sent — the user reviews, then sends. */
  prompt?: string;
}

/**
 * Start a fresh coder session pointed at a workspace and navigate to the coder
 * page. The shared entry point behind every "Open in Coder" action. The pending
 * prompt is consumed by `CoderPanel` on mount.
 */
export function openCoderSession(opts: OpenCoderSessionOptions = {}): void {
  const { workspaceRoot, projectId = null, prompt } = opts;
  if (workspaceRoot) coderUi.setActiveProject(workspaceRoot, projectId);
  coderSession.newSession();
  if (workspaceRoot) coderSession.workspaceRoot = workspaceRoot;
  coderSession.pendingPrompt = prompt ?? null;
  void goto('/ai/coder');
}
