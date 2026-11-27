import { invokeClient } from '$lib/utils/invokeClient';
import type { DriveProjectStatus } from '../generated/DriveProjectStatus';
import type { DriveSyncDirection } from '../generated/DriveSyncDirection';
import type { DriveSyncResult } from '../generated/DriveSyncResult';
import type { SyncDrive } from '../generated/SyncDrive';

export async function fetchSyncDrives(): Promise<SyncDrive[]> {
  return (await invokeClient.post<SyncDrive[]>('drive_sync_drives')) ?? [];
}

export function fetchDriveStatus(drive: string, projectId: number): Promise<DriveProjectStatus> {
  return invokeClient.post<DriveProjectStatus>('drive_sync_status', { drive, projectId });
}

export function applyDriveSync(
  drive: string,
  projectId: number,
  direction: DriveSyncDirection,
  force: boolean
): Promise<DriveSyncResult> {
  return invokeClient.post<DriveSyncResult>('drive_sync_apply', {
    drive,
    projectId,
    direction,
    force,
  });
}
