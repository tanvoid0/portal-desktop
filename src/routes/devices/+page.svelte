<!--
	Devices Page
	Pair, approve and manage phones connecting over the Android remote-control protocol.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { PageEmpty, PageError, PageHeader, PageLoading, PageStats } from '$lib/components/shell';
  import type { PageStat } from '$lib/components/shell';
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

  const stats = $derived<PageStat[]>([
    { label: 'Pending', value: pending.length, icon: ClockIcon },
    { label: 'Paired', value: paired.length, icon: MobileIcon },
  ]);

  function refetchDevices() {
    return queryClient.invalidateQueries({ queryKey: queryKeys.devices.all });
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

<div class="space-y-6">
  <PageHeader title="Devices" description="Pair and manage phones connecting to Portal Desktop">
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={refetchDevices} disabled={devicesQ.isFetching}>
        <RefreshCw class="mr-2 h-4 w-4 {devicesQ.isFetching ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    {/snippet}
  </PageHeader>

  {#if devicesQ.isPending}
    <PageLoading message="Loading devices..." />
  {:else if devicesQ.error}
    <PageError
      message={devicesQ.error instanceof Error ? devicesQ.error.message : 'Failed to load devices'}
      onRetry={refetchDevices}
    />
  {:else}
    <PageStats {stats} />

    <LanToggleCard />

    {#if showConnectivityWarning}
      <FirewallWarningBanner {firewallRuleMissing} {vpnAdapter} />
    {/if}

    <PairDeviceCard {capabilities} onPaired={refetchDevices} />

    {#if pending.length > 0}
      <div class="space-y-3">
        <h3 class="text-sm font-semibold">Pending requests</h3>
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {#each pending as device (device.device_id)}
            <PendingRequestRow {device} {capabilities} onResolved={refetchDevices} />
          {/each}
        </div>
      </div>
    {/if}

    <div class="space-y-3">
      <h3 class="text-sm font-semibold">Paired devices</h3>
      {#if paired.length === 0}
        <PageEmpty
          title="No paired devices"
          description="Generate a QR code above to pair a phone."
          icon={MobileIcon}
        />
      {:else}
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {#each paired as device (device.device_id)}
            <DeviceRow {device} {capabilities} onChanged={refetchDevices} />
          {/each}
        </div>
      {/if}
    </div>

    {#if history.length > 0}
      <div class="space-y-3">
        <Button variant="ghost" size="sm" onclick={() => (historyOpen = !historyOpen)}>
          {historyOpen ? 'Hide' : 'Show'} history ({history.length})
        </Button>
        {#if historyOpen}
          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {#each history as device (device.device_id)}
              <DeviceRow {device} {capabilities} onChanged={refetchDevices} />
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</div>
