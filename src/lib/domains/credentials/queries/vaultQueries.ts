import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchVaultStatus } from '../api/vaultApi';

/** Vault lock/setup state. Refetches on focus so a lock/unlock from another window shows up promptly. */
export function createVaultStatusQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.credentials.status,
    queryFn: fetchVaultStatus,
    refetchOnWindowFocus: true,
  }));
}
