import { goto } from '$app/navigation';
import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
import { coderUi } from '$lib/domains/coder/state/coderUi.svelte';
import { toast } from '$lib/utils/toast';
import type { Project } from '$lib/domains/projects/types';

export async function openInExplorer(path: string): Promise<void> {
  try {
    const { openPath } = await import('@tauri-apps/plugin-opener');
    await openPath(path);
  } catch {
    toast.error('Could not open the folder');
  }
}

/** Points the Coder page at this project's workspace and switches to it. */
export function launchInCoder(project: Project): void {
  coderUi.setActiveProject(project.path, project.id);
  coderSession.newSession();
  coderSession.workspaceRoot = project.path;
  void goto('/ai/coder');
}
