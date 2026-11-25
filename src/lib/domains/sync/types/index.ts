export interface SyncRecord {
  version: number;
  deviceId: string;
  clientUpdatedAt: string;
  size: number;
  contentHash: string;
  createdAt: string;
}

export interface SyncConfig {
  serverUrl: string;
  hasToken: boolean;
}

export interface SyncStatus {
  configured: boolean;
  deviceId: string;
  baseVersion: number;
  lastSyncedAt: string | null;
  localDbModifiedAt: string | null;
  localDbSize: number;
  cloud: SyncRecord | null;
  cloudError: string | null;
}

export interface LocalInfo {
  baseVersion: number;
  localDbModifiedAt: string | null;
  localDbSize: number;
}

export type PushResult =
  | { status: 'ok'; record: SyncRecord }
  | { status: 'conflict'; current: SyncRecord; local: LocalInfo };

export interface PullResult {
  status: 'restored' | 'restart_required';
  record: SyncRecord;
  unresolvedCount: number;
}

export interface UnresolvedProject {
  projectId: number;
  name: string;
  gitRemote: string | null;
  lastKnownPath: string | null;
  lastKnownDeviceId: string | null;
}

export type SyncVerdict =
  'in-sync' | 'cloud-newer' | 'local-newer' | 'diverged' | 'never' | 'unknown';
