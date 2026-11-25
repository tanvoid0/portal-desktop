import type {
  LocalInfo,
  PullResult,
  PushResult,
  SyncConfig,
  SyncRecord,
  SyncStatus,
  UnresolvedProject,
} from '../types';

type Raw = Record<string, unknown>;
const obj = (v: unknown): Raw => (v ?? {}) as Raw;

export function normalizeRecord(raw: unknown): SyncRecord {
  const r = obj(raw);
  return {
    version: Number(r.version ?? 0),
    deviceId: String(r.device_id ?? ''),
    clientUpdatedAt: String(r.client_updated_at ?? ''),
    size: Number(r.size ?? 0),
    contentHash: String(r.content_hash ?? ''),
    createdAt: String(r.created_at ?? ''),
  };
}

export function normalizeConfig(raw: unknown): SyncConfig {
  const r = obj(raw);
  return { serverUrl: String(r.server_url ?? ''), hasToken: Boolean(r.has_token) };
}

export function normalizeStatus(raw: unknown): SyncStatus {
  const r = obj(raw);
  return {
    configured: Boolean(r.configured),
    deviceId: String(r.device_id ?? ''),
    baseVersion: Number(r.base_version ?? 0),
    lastSyncedAt: (r.last_synced_at as string | null) ?? null,
    localDbModifiedAt: (r.local_db_modified_at as string | null) ?? null,
    localDbSize: Number(r.local_db_size ?? 0),
    cloud: r.cloud ? normalizeRecord(r.cloud) : null,
    cloudError: (r.cloud_error as string | null) ?? null,
  };
}

function normalizeLocal(raw: unknown): LocalInfo {
  const r = obj(raw);
  return {
    baseVersion: Number(r.base_version ?? 0),
    localDbModifiedAt: (r.local_db_modified_at as string | null) ?? null,
    localDbSize: Number(r.local_db_size ?? 0),
  };
}

export function normalizePush(raw: unknown): PushResult {
  const r = obj(raw);
  if (r.status === 'conflict') {
    return {
      status: 'conflict',
      current: normalizeRecord(r.current),
      local: normalizeLocal(r.local),
    };
  }
  return { status: 'ok', record: normalizeRecord(r.record) };
}

export function normalizePull(raw: unknown): PullResult {
  const r = obj(raw);
  return {
    status: r.status === 'restart_required' ? 'restart_required' : 'restored',
    record: normalizeRecord(r.record),
    unresolvedCount: Number(r.unresolved_count ?? 0),
  };
}

export function normalizeUnresolved(raw: unknown): UnresolvedProject {
  const r = obj(raw);
  return {
    projectId: Number(r.project_id),
    name: String(r.name ?? ''),
    gitRemote: (r.git_remote as string | null) ?? null,
    lastKnownPath: (r.last_known_path as string | null) ?? null,
    lastKnownDeviceId: (r.last_known_device_id as string | null) ?? null,
  };
}
