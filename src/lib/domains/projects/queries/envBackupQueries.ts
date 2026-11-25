import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { scanEnvBackup } from '../api/envBackupApi';

/** Env files vs. their vault copies. Refetches on focus so edits made in an editor show up. */
export function createEnvBackupScanQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.envBackup,
    queryFn: scanEnvBackup,
    refetchOnWindowFocus: true,
  }));
}
