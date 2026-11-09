import { invokeClient } from '$lib/utils/invokeClient';
import type { VaultSetupResult, VaultStatus } from '../types';

/** Map a raw Rust `VaultStatusResponse` (snake_case) to the TS shape. */
function normalizeVaultStatus(raw: unknown): VaultStatus {
  const r = (raw ?? {}) as Record<string, unknown>;
  return {
    state: r.state as VaultStatus['state'],
    helloAvailable: Boolean(r.hello_available),
    hasCredentials: Boolean(r.has_credentials),
  };
}

export async function fetchVaultStatus(): Promise<VaultStatus> {
  return normalizeVaultStatus(await invokeClient.post<unknown>('vault_status'));
}

export async function setupVault(password: string): Promise<VaultSetupResult> {
  const raw = await invokeClient.post<{ recovery_key: string }>('vault_setup', { password });
  return { recoveryKey: raw.recovery_key };
}

export async function unlockVaultWithPassword(password: string): Promise<void> {
  await invokeClient.post<void>('vault_unlock_with_password', { password });
}

export async function unlockVaultWithHello(): Promise<void> {
  await invokeClient.post<void>('vault_unlock_with_hello');
}

export async function lockVault(): Promise<void> {
  await invokeClient.post<void>('vault_lock');
}

export async function resetVaultPassword(recoveryKey: string, newPassword: string): Promise<void> {
  await invokeClient.post<void>('vault_reset_password', {
    recoveryKey,
    newPassword,
  });
}
