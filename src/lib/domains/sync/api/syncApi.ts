// Whole backend surface for online sync. Contract changes land here only.
import { invokeClient } from '$lib/utils/invokeClient';
import {
  normalizeConfig,
  normalizePull,
  normalizePush,
  normalizeRecord,
  normalizeStatus,
  normalizeUnresolved,
} from '../utils/normalizeSync';
import type {
  PullResult,
  PushResult,
  SyncConfig,
  SyncRecord,
  SyncStatus,
  UnresolvedProject,
} from '../types';

export async function getSyncConfig(): Promise<SyncConfig> {
  return normalizeConfig(await invokeClient.post<unknown>('sync_get_config'));
}

/** `token` omitted keeps the stored one. */
export async function setSyncConfig(serverUrl: string, token?: string): Promise<void> {
  await invokeClient.post<void>('sync_set_config', { serverUrl, token });
}

/** Stores access + refresh tokens; the password is never kept. */
export async function syncLogin(serverUrl: string, email: string, password: string): Promise<void> {
  await invokeClient.post<void>('sync_login', { serverUrl, email, password });
}

export async function getSyncStatus(): Promise<SyncStatus> {
  return normalizeStatus(await invokeClient.post<unknown>('sync_status'));
}

export async function listSyncVersions(): Promise<SyncRecord[]> {
  const raw = await invokeClient.post<unknown[]>('sync_list_versions');
  return (raw ?? []).map(normalizeRecord);
}

export async function pushSync(password: string, force: boolean): Promise<PushResult> {
  return normalizePush(await invokeClient.post<unknown>('sync_push', { password, force }));
}

export async function pullSync(password: string, version?: number): Promise<PullResult> {
  return normalizePull(await invokeClient.post<unknown>('sync_pull', { password, version }));
}

export async function listUnresolvedProjects(): Promise<UnresolvedProject[]> {
  const raw = await invokeClient.post<unknown[]>('sync_list_unresolved_projects');
  return (raw ?? []).map(normalizeUnresolved);
}

export async function resolveProjectPath(projectId: number, path: string): Promise<void> {
  await invokeClient.post<void>('sync_resolve_project_path', { projectId, path });
}

export async function ignoreProject(projectId: number): Promise<void> {
  await invokeClient.post<void>('sync_ignore_project', { projectId });
}
