import { invoke } from '@tauri-apps/api/core';
import { openCoderSession } from '$lib/domains/coder/utils/launch';
import { openAgentsSession } from '$lib/domains/agents/utils/launch';
import { toast } from '$lib/utils/toast';
import type { Project } from '$lib/domains/projects/types';

/** Opens a folder in the OS file manager. Goes through Rust because the
 *  webview's `openPath` is not granted by `opener:default`. */
export async function openInExplorer(path: string): Promise<void> {
  try {
    await invoke('coder_open_in_explorer', { workspaceRoot: path, path: '.' });
  } catch (err) {
    toast.error('Could not open the folder', err);
  }
}

/** Points the Coder page at this project's workspace and switches to it. */
export function launchInCoder(project: Project, prompt?: string): void {
  openCoderSession({ workspaceRoot: project.path, projectId: project.id, prompt });
}

/** Points the Agents page at this project's workspace and switches to it. */
export function launchInAgents(project: Project, prompt?: string): void {
  openAgentsSession({ cwd: project.path, prompt });
}
