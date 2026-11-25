import { describe, expect, it } from 'vitest';
import { syncVerdict } from './syncVerdict';
import type { SyncStatus } from '../types';

const rec = (version: number) => ({
  version,
  deviceId: 'd',
  clientUpdatedAt: '',
  size: 1,
  contentHash: '',
  createdAt: '',
});
const base: SyncStatus = {
  configured: true,
  deviceId: 'a',
  baseVersion: 2,
  lastSyncedAt: '2026-01-02T00:00:00Z',
  localDbModifiedAt: '2026-01-01T00:00:00Z',
  localDbSize: 1,
  cloud: rec(2),
  cloudError: null,
};
const edited = '2026-01-03T00:00:00Z';

describe('syncVerdict', () => {
  it('in sync', () => expect(syncVerdict(base)).toBe('in-sync'));
  it('cloud newer', () => expect(syncVerdict({ ...base, cloud: rec(3) })).toBe('cloud-newer'));
  it('local newer', () =>
    expect(syncVerdict({ ...base, localDbModifiedAt: edited })).toBe('local-newer'));
  it('diverged', () =>
    expect(syncVerdict({ ...base, cloud: rec(3), localDbModifiedAt: edited })).toBe('diverged'));
  it('never / unknown', () => {
    expect(syncVerdict({ ...base, baseVersion: 0 })).toBe('never');
    expect(syncVerdict({ ...base, cloud: null })).toBe('never');
    expect(syncVerdict({ ...base, cloud: null, cloudError: 'offline' })).toBe('unknown');
  });
});

describe('syncVerdict null local mtime', () => {
  it('treats missing mtime as unchanged', () =>
    expect(syncVerdict({ ...base, localDbModifiedAt: null })).toBe('in-sync'));
});
