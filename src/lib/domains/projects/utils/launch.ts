import { openCoderSession } from '$lib/domains/coder/utils/launch';
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
export function launchInCoder(project: Project, prompt?: string): void {
  openCoderSession({ workspaceRoot: project.path, projectId: project.id, prompt });
}
