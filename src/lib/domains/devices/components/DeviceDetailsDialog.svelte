<script lang="ts">
  import { untrack } from 'svelte';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as RadioGroup from '$lib/components/ui/radio-group';
  import { Label } from '$lib/components/ui/label';
  import { Button } from '$lib/components/ui/button';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { setDeviceLifetime, updateDeviceCapabilities } from '../api/deviceApi';
  import { toast } from '$lib/utils/toast';
  import { sameCapabilities } from '../utils/duplicateDevices';
  import type { ApprovalType, CapabilityInfo, DeviceSummary } from '../types';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    device: DeviceSummary;
    capabilities: CapabilityInfo[];
    open: boolean;
    onChanged?: () => void;
  }

  let { device, capabilities, open = $bindable(false), onChanged = () => {} }: Props = $props();

  const LIFETIMES: Array<{ value: ApprovalType; label: string }> = [
    { value: 'temporary', label: '24 hours' },
    { value: 'long_term', label: '30 days' },
    { value: 'until_revoked', label: 'Until revoked' },
  ];
  /** Lifetimes the backend can switch between (`set_lifetime` in
   * pairing_service.rs); a QR-minted or pending grant isn't one of them. */
  const EDITABLE = new Set(['temporary', 'long_term', 'until_revoked']);

  let selected = $state<string[]>([...device.capabilities]);
  let lifetime = $state<ApprovalType>(device.approval_type as ApprovalType);
  let busy = $state(false);

  // Reopening on a device whose grant changed elsewhere must not show stale
  // edits, so reset the form every time the dialog opens — and only then: a
  // refetch while it is open must not wipe what the user has edited.
  $effect(() => {
    if (!open) return;
    untrack(() => {
      selected = [...device.capabilities];
      lifetime = device.approval_type as ApprovalType;
    });
  });

  const expired = $derived(
    !!device.expires_at && new Date(device.expires_at).getTime() <= Date.now()
  );
  const statusLine = $derived.by(() => {
    if (!device.expires_at) return 'Access does not expire';
    const at = new Date(device.expires_at);
    return expired
      ? `Expired ${at.toLocaleString()} — pick a lifetime below to restore access`
      : `Expires ${at.toLocaleString()}`;
  });

  const canEditLifetime = $derived(device.approved && EDITABLE.has(device.approval_type));
  // An expired grant renews on the lifetime it already has — Save restarts the window.
  const lifetimeChanged = $derived(
    canEditLifetime && (lifetime !== device.approval_type || expired)
  );
  const capsChanged = $derived(!sameCapabilities(selected, device.capabilities));
  const dirty = $derived(lifetimeChanged || capsChanged);

  async function save() {
    busy = true;
    try {
      if (capsChanged) await updateDeviceCapabilities(device.device_id, selected);
      if (lifetimeChanged) await setDeviceLifetime(device.device_id, lifetime);
      toast.success('Device updated', { description: device.device_name });
      open = false;
    } catch (err) {
      toast.error('Failed to update device', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
      // Even after a half-applied save (caps saved, lifetime not), so the list shows what stuck.
      onChanged();
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>{device.device_name}</Dialog.Title>
      <Dialog.Description>{statusLine}</Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4">
      <div class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground">Lifetime</p>
        {#if canEditLifetime}
          <RadioGroup.Root bind:value={lifetime} class="flex flex-wrap gap-4" disabled={busy}>
            {#each LIFETIMES as option (option.value)}
              <Label class="flex items-center gap-1.5 text-sm font-normal">
                <RadioGroup.Item value={option.value} />
                {option.label}
              </Label>
            {/each}
          </RadioGroup.Root>
          {#if lifetimeChanged}
            <p class="text-xs text-muted-foreground">
              Saving restarts the window from now — the phone keeps the credential it has.
            </p>
          {/if}
        {:else}
          <p class="text-sm text-muted-foreground">
            This grant's lifetime can't be changed — pair the device again instead.
          </p>
        {/if}
      </div>

      <div class="space-y-2">
        <p class="text-xs font-medium text-muted-foreground">Capabilities</p>
        <CapabilityChecklist {capabilities} bind:selected disabled={busy} />
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)} disabled={busy}>Cancel</Button>
      <Button onclick={save} disabled={busy || !dirty}>
        {#if busy}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
        Save
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
