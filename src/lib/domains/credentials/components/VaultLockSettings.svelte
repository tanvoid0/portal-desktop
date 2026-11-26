<!--
	When the vault locks itself again (`vault_set_lock_policy`).
-->

<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import SettingsSection from '$lib/domains/settings/components/shared/SettingsSection.svelte';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import Clock from '~icons/fa6-solid/clock';
  import { fetchVaultLockPolicy, setVaultLockPolicy } from '../api/vaultApi';
  import type { KeepUnlocked, VaultLockPolicy } from '../types';

  interface Props {
    unlocked: boolean;
  }
  let { unlocked }: Props = $props();

  const MAX_IDLE_MINUTES = 10080;

  const options: { value: KeepUnlocked; label: string; hint: string }[] = [
    {
      value: 'app_close',
      label: 'Lock when the app closes',
      hint: 'You unlock once each time you open the app.',
    },
    {
      value: 'system_restart',
      label: 'Lock when the computer restarts',
      hint: 'Stays unlocked across app restarts until you restart or sign out.',
    },
    {
      value: 'until_locked',
      label: 'Keep unlocked until I lock it',
      hint: 'Anyone using your signed-in account can use your saved credentials.',
    },
  ];

  const policyQuery = createQuery(() => ({
    queryKey: queryKeys.credentials.lockPolicy,
    queryFn: fetchVaultLockPolicy,
  }));
  const policy = $derived(policyQuery.data);
  const hint = $derived(options.find((o) => o.value === policy?.keepUnlocked)?.hint);

  async function save(next: VaultLockPolicy) {
    try {
      await setVaultLockPolicy(next);
    } catch (err) {
      toast.error(getErrorMessage(err, 'Failed to save the lock setting.'));
    }
    await queryClient.invalidateQueries({ queryKey: queryKeys.credentials.lockPolicy });
  }
</script>

<SettingsSection
  title="Auto-lock"
  description="Your password is never saved. Staying unlocked reads the vault key from your operating system's keychain."
  icon={Clock}
>
  {#if policy}
    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="vault-keep-unlocked">After unlocking</Label>
        <Select
          id="vault-keep-unlocked"
          {options}
          disabled={!unlocked}
          value={policy.keepUnlocked}
          onValueChange={(value: string) =>
            void save({ ...policy, keepUnlocked: value as KeepUnlocked })}
        />
        <p class="text-sm text-muted-foreground">{hint}</p>
      </div>
      <div class="space-y-2">
        <Label for="vault-idle-minutes">Also lock after idle minutes</Label>
        <Input
          id="vault-idle-minutes"
          type="number"
          min="0"
          max={MAX_IDLE_MINUTES}
          step="1"
          class="max-w-32"
          disabled={!unlocked}
          value={policy.idleMinutes}
          onchange={(e) => {
            const minutes = Math.floor(Number(e.currentTarget.value)) || 0;
            const idleMinutes = Math.min(Math.max(minutes, 0), MAX_IDLE_MINUTES);
            void save({ ...policy, idleMinutes });
          }}
        />
        <p class="text-sm text-muted-foreground">
          0 turns it off. Counts only while the app is running.
        </p>
      </div>
      {#if !unlocked}
        <p class="text-sm text-muted-foreground">Unlock the vault to change these.</p>
      {/if}
    </div>
  {:else if policyQuery.isError}
    <p class="text-sm text-destructive" role="alert">
      {getErrorMessage(policyQuery.error, 'Failed to load the lock settings.')}
    </p>
  {/if}
</SettingsSection>
