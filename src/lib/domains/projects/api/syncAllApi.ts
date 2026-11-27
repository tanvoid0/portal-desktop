import { invokeClient } from '$lib/utils/invokeClient';
import type { SyncAllOptions } from '../generated/SyncAllOptions';
import type { SyncAllReport } from '../generated/SyncAllReport';
import type { SyncRepoFacts } from '../generated/SyncRepoFacts';
import type { SyncResolution } from '../generated/SyncResolution';
import type { SyncResolveResult } from '../generated/SyncResolveResult';

export const SYNC_ALL_PROGRESS_EVENT = 'sync-all://progress';

export async function syncAllProjects(options: SyncAllOptions): Promise<SyncAllReport> {
  return invokeClient.post<SyncAllReport>('sync_all_projects', { options });
}

export async function cancelSyncAll(): Promise<void> {
  await invokeClient.post<unknown>('cancel_sync_all', {});
}

export async function syncRepoFacts(projectId: number): Promise<SyncRepoFacts> {
  return invokeClient.post<SyncRepoFacts>('sync_repo_facts', { projectId });
}

export async function syncApplyResolution(
  projectId: number,
  plan: SyncResolution
): Promise<SyncResolveResult> {
  return invokeClient.post<SyncResolveResult>('sync_apply_resolution', { projectId, plan });
}
