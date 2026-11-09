import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchVaultStatus } from '../api/vaultApi';

/** Shared state for the vault-unlock gate — mounted once via `VaultGate`. */
class VaultGateState {
  open = $state(false);
  /** Set by `VaultSetupDialog` once it's past the password step, so `VaultGate`
   *  keeps it mounted through the recovery-key reveal even if the status query
   *  refetches (setup unlocks the session immediately, which would otherwise
   *  flip the query to 'unlocked' and unmount the dialog mid-reveal). */
  setupStep: 'password' | 'recovery' | null = $state(null);
  private waiters: ((ok: boolean) => void)[] = [];

  /** Opens the gate and resolves once the vault is unlocked (true) or the user cancels (false).
   *  Concurrent callers all resolve together when `settle` runs. The cached vault-status query
   *  can be up to 5 minutes stale (or locked server-side after a 15-min idle timeout with no
   *  push to the frontend), so force a fresh check before deciding whether to open the gate. */
  async ensureUnlocked(): Promise<boolean> {
    const status = await queryClient.fetchQuery({
      queryKey: queryKeys.credentials.status,
      queryFn: fetchVaultStatus,
      staleTime: 0,
    });
    if (status.state === 'unlocked') return true;
    this.open = true;
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  settle(ok: boolean) {
    this.open = false;
    const waiters = this.waiters;
    this.waiters = [];
    waiters.forEach((resolve) => resolve(ok));
  }
}

export const vaultGate = new VaultGateState();
