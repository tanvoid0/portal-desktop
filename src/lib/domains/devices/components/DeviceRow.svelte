<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { revokeDevice, updateDeviceCapabilities } from '../api/deviceApi';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import type { DeviceSummary } from '../types';
  import Pencil from '~icons/fa6-solid/pen';
  import Loader2 from '~icons/fa6-solid/spinner';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';

  interface Props {
    device: DeviceSummary;
    capabilities: string[];
    duplicates?: DeviceSummary[];
    onChanged?: () => void;
  }

  let { device, capabilities, duplicates = [], onChanged = () => {} }: Props = $props();

  const APPROVAL_LABEL: Record<string, string> = {
    pending: 'Pending',
    temporary: 'Temporary (24h)',
    long_term: 'Long-term (30d)',
    until_revoked: 'Until revoked',
    revoked: 'Revoked',
    rejected: 'Rejected',
  };

  // A device is "active" if it has touched the daemon in the last 5 minutes.
  // There's no live connection tracking to say a device is online right now,
  // so this is the honest proxy: recent use.
  const ACTIVE_WINDOW_MS = 5 * 60 * 1000;

  let editing = $state(false);
  let selected = $state<string[]>([...device.capabilities]);
  let busy = $state(false);

  const isActive = $derived(device.approved && device.approval_type !== 'revoked');
  const lastUsed = $derived(device.last_used_at ? new Date(device.last_used_at) : null);
  const isOnline = $derived(
    isActive && !!lastUsed && Date.now() - lastUsed.getTime() < ACTIVE_WINDOW_MS
  );
  const lastSeenLabel = $derived(
    isOnline ? 'Active now' : lastUsed ? formatRelativeTime(lastUsed) : 'Never used'
  );
  const dotClass = $derived(
    isOnline ? 'bg-green-500' : isActive ? 'bg-muted-foreground/40' : 'bg-destructive/50'
  );

  async function saveCapabilities() {
    busy = true;
    try {
      await updateDeviceCapabilities(device.device_id, selected);
      toast.success('Capabilities updated');
      editing = false;
      onChanged();
    } catch (err) {
      toast.error(
        'Failed to update capabilities',
        err instanceof Error ? err.message : String(err)
      );
    } finally {
      busy = false;
    }
  }

  async function revoke(deviceId: string = device.device_id, name: string = device.device_name) {
    const confirmed = await confirmAction(
      `Revoke access for ${name}? It will have to be paired again.`,
      'Revoke device'
    );
    if (!confirmed) return;

    busy = true;
    try {
      await revokeDevice(deviceId);
      toast.success('Device revoked');
      onChanged();
    } catch (err) {
      toast.error('Failed to revoke device', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  }
</script>

<Card class="flex flex-col gap-2 p-3">
  <div class="flex items-start justify-between gap-2">
    <div class="flex min-w-0 items-center gap-2">
      <span
        class="h-2 w-2 shrink-0 rounded-full {dotClass}"
        title={lastSeenLabel}
        aria-hidden="true"
      ></span>
      <span class="truncate text-sm font-medium" title={device.device_name}>
        {device.device_name}
      </span>
    </div>
    <Badge variant={isActive ? 'default' : 'outline'} class="shrink-0 font-normal">
      {APPROVAL_LABEL[device.approval_type] ?? device.approval_type}
    </Badge>
  </div>

  {#if duplicates.length > 0}
    <div class="flex flex-col gap-1 rounded-md bg-destructive/10 p-2 text-xs text-destructive">
      {#each duplicates as other (other.device_id)}
        <div class="flex items-center justify-between gap-2">
          <span class="flex items-center gap-1.5 truncate">
            <TriangleExclamation class="h-3 w-3 shrink-0" />
            Also paired as "{other.device_name}" · {APPROVAL_LABEL[other.approval_type] ??
              other.approval_type} · {other.capabilities.join(', ') || 'no capabilities'}
          </span>
          <Button
            variant="outline"
            size="sm"
            class="h-6 shrink-0 px-2 text-xs"
            onclick={() => revoke(other.device_id, other.device_name)}
            disabled={busy}
          >
            Archive old
          </Button>
        </div>
      {/each}
    </div>
  {/if}

  {#if editing}
    <CapabilityChecklist {capabilities} bind:selected disabled={busy} />
    <div class="flex gap-2">
      <Button size="sm" onclick={saveCapabilities} disabled={busy}>Save</Button>
      <Button size="sm" variant="outline" onclick={() => (editing = false)} disabled={busy}>
        Cancel
      </Button>
    </div>
  {:else}
    <div class="flex flex-wrap items-center gap-1">
      {#each device.capabilities as cap}
        <Badge variant="secondary" class="text-xs font-normal">{cap}</Badge>
      {:else}
        <span class="text-xs text-muted-foreground">no capabilities</span>
      {/each}
      {#if isActive}
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5"
          onclick={() => {
            selected = [...device.capabilities];
            editing = true;
          }}
          aria-label="Edit capabilities"
        >
          <Pencil class="h-3 w-3" />
        </Button>
      {/if}
    </div>

    <div class="flex items-center justify-between text-xs text-muted-foreground">
      <span>{lastSeenLabel}</span>
      {#if device.expires_at}
        <span>Expires {new Date(device.expires_at).toLocaleDateString()}</span>
      {/if}
    </div>

    {#if isActive}
      <Button
        variant="destructive"
        size="sm"
        onclick={() => revoke()}
        disabled={busy}
        class="self-end"
      >
        {#if busy}
          <Loader2 class="h-4 w-4 animate-spin" />
        {:else}
          Revoke
        {/if}
      </Button>
    {/if}
  {/if}
</Card>
