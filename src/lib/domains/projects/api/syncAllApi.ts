import { invokeClient } from '$lib/utils/invokeClient';
import type { SyncAllOptions } from '../generated/SyncAllOptions';
import type { SyncAllReport } from '../generated/SyncAllReport';

export const SYNC_ALL_PROGRESS_EVENT = 'sync-all://progress';

export async function syncAllProjects(options: SyncAllOptions): Promise<SyncAllReport> {
  return invokeClient.post<SyncAllReport>('sync_all_projects', { options });
}

export async function cancelSyncAll(): Promise<void> {
  await invokeClient.post<unknown>('cancel_sync_all', {});
}
