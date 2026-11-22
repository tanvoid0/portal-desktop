<!--
	Devices Page
	Pair, approve and manage phones connecting over the Android remote-control protocol.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { PageEmpty, PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { devicesUi } from '$lib/domains/devices/state/devicesUi.svelte';
  import LanToggleCard from '$lib/domains/devices/components/LanToggleCard.svelte';
  import PairDeviceCard from '$lib/domains/devices/components/PairDeviceCard.svelte';
  import PendingRequestRow from '$lib/domains/devices/components/PendingRequestRow.svelte';
  import DeviceRow from '$lib/domains/devices/components/DeviceRow.svelte';
  import FirewallWarningBanner from '$lib/domains/devices/components/FirewallWarningBanner.svelte';
  import {
    createCapabilitiesQuery,
    createConnectivityStatusQuery,
    createDevicesQuery,
  } from '$lib/domains/devices/queries/deviceQueries';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { findDuplicatesByName } from '$lib/domains/devices/utils/duplicateDevices';
  import type { DeviceSummary } from '$lib/domains/devices/types';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ClockIcon from '~icons/fa6-solid/clock';
  import MobileIcon from '~icons/fa6-solid/mobile-screen-button';

  const queryClient = useQueryClient();
  const devicesQ = createDevicesQuery();
  const capabilitiesQ = createCapabilitiesQuery();
  const connectivityQ = createConnectivityStatusQuery();

  const lanOn = $derived(connectivityQ.data?.lan_enabled === true);
  const firewallRuleMissing = $derived(
    lanOn && connectivityQ.data?.firewall_rule_present === false
  );
  const vpnAdapter = $derived(lanOn ? (connectivityQ.data?.vpn_adapter ?? null) : null);
  const showConnectivityWarning = $derived(firewallRuleMissing || vpnAdapter !== null);

  const capabilities = $derived(capabilitiesQ.data ?? []);
  const devices = $derived(devicesQ.data ?? []);
  const pending = $derived(devices.filter((d) => d.approval_type === 'pending'));
  // A QR-minted grant keeps its pickup `expires_at` until the first scan
  // clears it (see `mint_qr_pairing`/`verify_token` in pairing_service.rs) —
  // so `until_revoked` rows that still carry one haven't been picked up yet,
  // whether or not the 5-minute window has lapsed. Per the QR card's own
  // copy ("the grant appears below once scanned"), those don't belong in
  // "Paired devices" or its count.
  const paired = $derived(
    devices.filter(
      (d) =>
        d.approved &&
        d.approval_type !== 'revoked' &&
        d.approval_type !== 'rejected' &&
        !(d.approval_type === 'until_revoked' && d.expires_at !== null)
    )
  );
  const history = $derived(
    devices.filter((d) => d.approval_type === 'revoked' || d.approval_type === 'rejected')
  );
  const pairedDeviceIds = $derived(new Set(paired.map((d) => d.device_id)));
  // Same device_name among several live grants — the most likely cause is a
  // stale connection nobody revoked before pairing again, since device_id is
  // never stable across requests. Flag it so the user can clean up.
  const duplicatesByDeviceId = $derived.by(() => {
    const map = new Map<string, DeviceSummary[]>();
    for (const device of paired) {
      map.set(device.device_id, findDuplicatesByName(device, paired));
    }
    return map;
  });

  function refetchDevices() {
    return queryClient.invalidateQueries({ queryKey: queryKeys.devices.all });
  }

  // Distinct from `devicesQ.isFetching`, which also flips on QA-116's
  // background pairing poll every ~3s — this flag is only ever set by the
  // Refresh button's own click, so the button's busy state reflects a
  // user-triggered refresh, not the poll (QA-115).
  let refreshing = $state(false);
  async function handleRefreshClick() {
    refreshing = true;
    try {
      await refetchDevices();
    } finally {
      refreshing = false;
    }
  }

  let historyOpen = $state(false);

  let unlisten: UnlistenFn | undefined;
  listen('device://pending', () => {
    refetchDevices();
  }).then((fn) => {
    unlisten = fn;
  });
  onDestroy(() => unlisten?.());
</script>

<svelte:head>
  <title>Devices - Portal Desktop</title>
</svelte:head>

<div class="space-y-4">
  <PageHeader title="Devices" description="Phones connecting to Portal Desktop">
    {#snippet actions()}
      <Label
        class="flex items-center gap-2 text-xs font-normal text-muted-foreground"
        title="Show each device's individual permissions and the full permission checklist"
      >
        <Switch
          checked={devicesUi.advanced}
          onCheckedChange={(v) => devicesUi.setAdvanced(v === true)}
        />
        Advanced
      </Label>
      <Button
        variant="outline"
        size="sm"
        onclick={handleRefreshClick}
        disabled={refreshing}
        aria-label="Refresh devices"
      >
        <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
      </Button>
    {/snippet}
  </PageHeader>

  {#if devicesQ.isPending}
    <PageLoading message="Loading devices..." />
  {:else if devicesQ.error && !devicesQ.data}
    <PageError
      message={devicesQ.error instanceof Error ? devicesQ.error.message : 'Failed to load devices'}
      onRetry={refetchDevices}
    />
  {:else}
    <LanToggleCard compact={!devicesUi.advanced} />

    {#if showConnectivityWarning}
      <FirewallWarningBanner {firewallRuleMissing} {vpnAdapter} />
    {/if}

    {#if pending.length > 0}
      <div class="space-y-2">
        <h3 class="flex items-center gap-2 text-sm font-semibold">
          <ClockIcon class="h-3.5 w-3.5 text-muted-foreground" />
          Waiting for you ({pending.length})
        </h3>
        <div class="space-y-2">
          {#each pending as device (device.device_id)}
            <PendingRequestRow
              {device}
              {capabilities}
              existing={findDuplicatesByName(device, paired)[0]}
              onResolved={refetchDevices}
            />
          {/each}
        </div>
      </div>
    {/if}

    <PairDeviceCard {capabilities} {pairedDeviceIds} onPaired={refetchDevices} />

    <div class="space-y-2">
      <h3 class="text-sm font-semibold">Paired devices ({paired.length})</h3>
      {#if paired.length === 0}
        <PageEmpty
          title="No paired devices"
          description="Generate a QR code above and scan it with the phone."
          icon={MobileIcon}
        />
      {:else}
        <div class="space-y-2">
          {#each paired as device (device.device_id)}
            <DeviceRow
              {device}
              {capabilities}
              duplicates={duplicatesByDeviceId.get(device.device_id) ?? []}
              onChanged={refetchDevices}
            />
          {/each}
        </div>
      {/if}
    </div>

    {#if history.length > 0}
      <div class="space-y-2">
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2 text-xs text-muted-foreground"
          onclick={() => (historyOpen = !historyOpen)}
        >
          {historyOpen ? 'Hide' : 'Show'} past devices ({history.length})
        </Button>
        {#if historyOpen}
          <!-- Nothing is actionable on a revoked or rejected grant, so these are
               plain text lines rather than the full device card. -->
          <ul class="divide-y rounded-md border text-xs">
            {#each history as device (device.device_id)}
              <li class="flex items-center justify-between gap-3 px-3 py-1.5">
                <span class="truncate font-medium" title={device.device_name}>
                  {device.device_name}
                </span>
                <span class="shrink-0 text-muted-foreground">
                  {device.approval_type === 'rejected' ? 'Denied' : 'Revoked'} ·
                  {formatRelativeTime(new Date(device.last_used_at ?? device.created_at))}
                </span>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  {/if}
</div>
