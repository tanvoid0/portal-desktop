import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { fetchVaultStatus } from '../api/vaultApi';

/** Shared state for the vault-unlock gate — mounted once via `VaultGate`. */
class VaultGateState {
  open = $state(false);
  /** Why the dialog is open, shown in it. First caller wins while several wait. */
  reason = $state('');
  /** Set by `VaultSetupDialog` once it's past the password step, so `VaultGate`
   *  keeps it mounted through the recovery-key reveal even if the status query
   *  refetches (setup unlocks the session immediately, which would otherwise
   *  flip the query to 'unlocked' and unmount the dialog mid-reveal). */
  setupStep: 'password' | 'recovery' | null = $state(null);
  private waiters: ((ok: boolean) => void)[] = [];

  /** Opens the gate and resolves once the vault is unlocked (true) or the user cancels (false).
   *  Concurrent callers all resolve together when `settle` runs. The cached vault-status query
   *  can be up to 5 minutes stale (another window may have locked it), so force a fresh check
   *  before deciding whether to open the gate. */
  async ensureUnlocked(reason = ''): Promise<boolean> {
    const status = await queryClient.fetchQuery({
      queryKey: queryKeys.credentials.status,
      queryFn: fetchVaultStatus,
      staleTime: 0,
    });
    if (status.state === 'unlocked') return true;
    if (!this.open) this.reason = reason;
    this.open = true;
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  /** Startup: if the vault has credentials but is locked, ask once so the key is held for the
   *  rest of the run (the backend has no idle lock). Skips needs_setup — nothing to unlock. */
  async promptIfLocked(): Promise<void> {
    const status = await fetchVaultStatus();
    if (status.state === 'locked' && status.hasCredentials)
      await this.ensureUnlocked('Unlock to use your saved credentials.');
  }

  settle(ok: boolean) {
    this.open = false;
    this.reason = '';
    const waiters = this.waiters;
    this.waiters = [];
    waiters.forEach((resolve) => resolve(ok));
  }
}

export const vaultGate = new VaultGateState();
