<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import IconAction from './IconAction.svelte';
  import DeviceDetailsDialog from './DeviceDetailsDialog.svelte';
  import { extendDeviceExpiry, revokeDevice } from '../api/deviceApi';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { devicesUi } from '../state/devicesUi.svelte';
  import type { CapabilityInfo, DeviceSummary } from '../types';
  import ReconnectQrDialog from './ReconnectQrDialog.svelte';
  import Pencil from '~icons/fa6-solid/pen';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Ban from '~icons/fa6-solid/ban';
  import ArrowRotateRight from '~icons/fa6-solid/arrow-rotate-right';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';

  interface Props {
    device: DeviceSummary;
    capabilities: CapabilityInfo[];
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

  let detailsOpen = $state(false);
  let busy = $state(false);

  const HOUR_MS = 60 * 60 * 1000;

  const isActive = $derived(device.approved && device.approval_type !== 'revoked');
  // `list_devices` keeps an expired temporary/long_term grant in the list, and
  // the daemon refuses it (`is_expired` in pairing_service.rs) — so the row has
  // to say so, or it reads as a working device that silently can't connect.
  const expiresAt = $derived(device.expires_at ? new Date(device.expires_at) : null);
  const expired = $derived(isActive && !!expiresAt && expiresAt.getTime() <= Date.now());
  // "Soon" scales with the grant: a 24h one is inside a one-day window from the
  // moment it is approved, so it only counts for its last 2h.
  const soonMs = $derived((device.approval_type === 'temporary' ? 2 : 24) * HOUR_MS);
  const expiringSoon = $derived(
    isActive && !!expiresAt && !expired && expiresAt.getTime() - Date.now() < soonMs
  );
  const lastUsed = $derived(device.last_used_at ? new Date(device.last_used_at) : null);
  const isOnline = $derived(
    isActive && !expired && !!lastUsed && Date.now() - lastUsed.getTime() < ACTIVE_WINDOW_MS
  );
  const lastSeenLabel = $derived(
    isOnline ? 'Active now' : lastUsed ? formatRelativeTime(lastUsed) : 'Never used'
  );
  const expiryLabel = $derived.by(() => {
    if (!expiresAt) return null;
    if (expired) return `Expired ${formatRelativeTime(expiresAt)}`;
    if (expiringSoon) return 'Expires today';
    return `Expires ${expiresAt.toLocaleDateString()}`;
  });
  const dotClass = $derived(
    isOnline
      ? 'bg-green-500'
      : expired
        ? 'bg-destructive'
        : isActive
          ? 'bg-muted-foreground/40'
          : 'bg-destructive/50'
  );

  const canExtend = $derived(
    device.approval_type === 'temporary' || device.approval_type === 'long_term'
  );

  async function extend() {
    busy = true;
    try {
      const expiry = await extendDeviceExpiry(device.device_id);
      toast.success(
        'Access extended',
        expiry ? `Now expires ${new Date(expiry).toLocaleDateString()}` : undefined
      );
      onChanged();
    } catch (err) {
      toast.error('Failed to extend access', err instanceof Error ? err.message : String(err));
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

<Card class="flex flex-col gap-2 px-3 py-2">
  <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
    <div class="flex min-w-0 flex-1 items-center gap-2">
      <span
        class="h-2 w-2 shrink-0 rounded-full {dotClass}"
        title={lastSeenLabel}
        aria-hidden="true"
      ></span>
      <span class="truncate text-sm font-medium" title={device.device_name}>
        {device.device_name}
      </span>
      <span class="shrink-0 text-xs text-muted-foreground">{lastSeenLabel}</span>
      {#if expiryLabel}
        <span
          class="shrink-0 text-xs {expired || expiringSoon
            ? 'font-medium text-destructive'
            : 'text-muted-foreground'}"
        >
          · {expiryLabel}
        </span>
      {/if}
    </div>

    {#if devicesUi.advanced}
      <Badge variant={isActive ? 'default' : 'outline'} class="shrink-0 font-normal">
        {APPROVAL_LABEL[device.approval_type] ?? device.approval_type}
      </Badge>
    {/if}

    {#if isActive}
      <div class="flex shrink-0 items-center gap-0.5">
        <!-- Extend is the quick one-click renew; the expiry itself is edited in
             the details dialog. Simple mode only shows it once the grant has run
             out (or runs out today), i.e. once the phone has stopped working. -->
        {#if canExtend && (devicesUi.advanced || expired || expiringSoon)}
          <IconAction
            label={expired ? 'Renew access' : 'Extend access'}
            variant={expired || expiringSoon ? 'default' : 'ghost'}
            onclick={expired ? () => (detailsOpen = true) : extend}
            disabled={busy}
          >
            {#snippet icon()}
              <ArrowRotateRight class="h-3.5 w-3.5" />
            {/snippet}
          </IconAction>
        {/if}
        <ReconnectQrDialog {device} {onChanged} />
        <IconAction
          label="Edit access and capabilities"
          onclick={() => (detailsOpen = true)}
          disabled={busy}
        >
          {#snippet icon()}
            <Pencil class="h-3.5 w-3.5" />
          {/snippet}
        </IconAction>
        <IconAction
          label="Revoke access"
          class="text-destructive hover:text-destructive"
          onclick={() => revoke()}
          disabled={busy}
        >
          {#snippet icon()}
            {#if busy}
              <Loader2 class="h-3.5 w-3.5 animate-spin" />
            {:else}
              <Ban class="h-3.5 w-3.5" />
            {/if}
          {/snippet}
        </IconAction>
      </div>
    {/if}
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

  {#if devicesUi.advanced}
    <div class="flex flex-wrap items-center gap-1">
      {#each device.capabilities as cap}
        <Badge variant="secondary" class="text-xs font-normal">{cap}</Badge>
      {:else}
        <span class="text-xs text-muted-foreground">No permissions granted</span>
      {/each}
    </div>
  {/if}
</Card>

<DeviceDetailsDialog {device} {capabilities} {onChanged} bind:open={detailsOpen} />
