import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import {
  getSyncConfig,
  getSyncStatus,
  listSyncVersions,
  listUnresolvedProjects,
} from '../api/syncApi';

export const createSyncConfigQuery = () =>
  createQuery(() => ({ queryKey: queryKeys.sync.config, queryFn: getSyncConfig }));

export const createSyncStatusQuery = () =>
  createQuery(() => ({
    queryKey: queryKeys.sync.status,
    queryFn: getSyncStatus,
    refetchOnWindowFocus: true,
  }));

export const createSyncVersionsQuery = (enabled: () => boolean) =>
  createQuery(() => ({
    queryKey: queryKeys.sync.versions,
    queryFn: listSyncVersions,
    enabled: enabled(),
  }));

export const createUnresolvedQuery = () =>
  createQuery(() => ({ queryKey: queryKeys.sync.unresolved, queryFn: listUnresolvedProjects }));
