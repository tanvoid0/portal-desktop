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
