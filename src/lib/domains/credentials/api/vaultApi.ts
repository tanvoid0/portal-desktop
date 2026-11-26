import { invokeClient } from '$lib/utils/invokeClient';
import type { KeepUnlocked, VaultLockPolicy, VaultSetupResult, VaultStatus } from '../types';

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

/** Verifies the current password, then re-wraps the master key under the new one. */
export async function changeVaultPassword(oldPassword: string, newPassword: string): Promise<void> {
  await invokeClient.post<void>('vault_change_password', { oldPassword, newPassword });
}

export async function fetchVaultLockPolicy(): Promise<VaultLockPolicy> {
  const raw = await invokeClient.post<{ keep_unlocked: KeepUnlocked; idle_minutes: number }>(
    'vault_get_lock_policy'
  );
  return { keepUnlocked: raw.keep_unlocked, idleMinutes: raw.idle_minutes };
}

export async function setVaultLockPolicy(policy: VaultLockPolicy): Promise<void> {
  await invokeClient.post<void>('vault_set_lock_policy', {
    policy: { keep_unlocked: policy.keepUnlocked, idle_minutes: policy.idleMinutes },
  });
}

/** Answer to the backend's `vault://unlock-requested`: releases commands waiting on the vault. */
export async function settleVaultUnlock(ok: boolean): Promise<void> {
  await invokeClient.post<void>('vault_unlock_settled', { ok });
}
