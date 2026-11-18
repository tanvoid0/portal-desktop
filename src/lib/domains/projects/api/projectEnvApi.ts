import { invokeClient } from '$lib/utils/invokeClient';
import type { EnvLinkView, ProjectEnv } from '../generated';

export async function fetchProjectEnv(projectId: number): Promise<ProjectEnv> {
  return invokeClient.post<ProjectEnv>('get_project_env', { projectId });
}

export async function linkProjectEnv(
  projectId: number,
  envKey: string,
  fileName: string,
  credentialId: string
): Promise<EnvLinkView> {
  return invokeClient.post<EnvLinkView>('link_project_env', {
    projectId,
    envKey,
    fileName,
    credentialId,
  });
}

export async function unlinkProjectEnv(linkId: number): Promise<void> {
  await invokeClient.post('unlink_project_env', { linkId });
}

export async function resyncProjectEnvLink(linkId: number): Promise<EnvLinkView> {
  return invokeClient.post<EnvLinkView>('resync_project_env_link', { linkId });
}

export async function addProjectEnvFile(projectId: number, path: string): Promise<string> {
  return invokeClient.post<string>('add_project_env_file', { projectId, path });
}

export async function removeProjectEnvFile(projectId: number, relPath: string): Promise<void> {
  await invokeClient.post('remove_project_env_file', { projectId, relPath });
}

/** `value: null` removes the key. Edits only the one line in that file. */
export async function setProjectEnvVar(
  projectId: number,
  fileName: string,
  key: string,
  value: string | null
): Promise<void> {
  await invokeClient.post('set_project_env_var', { projectId, fileName, key, value });
}
