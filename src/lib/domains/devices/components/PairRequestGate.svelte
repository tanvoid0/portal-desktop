<!--
  App-wide pairing prompt. A phone asking to pair blocks server-side for ~170s
  waiting on a human, so the answer (Allow / Deny, plus the advanced checklist)
  is put in front of the user wherever they are — not behind a toast that
  navigates to /devices. Esc / click-away just hides it; the request stays
  under "Waiting for you" on the Devices page.
-->
<script lang="ts">
  import { useQueryClient } from '@tanstack/svelte-query';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';
  import * as Dialog from '$lib/components/ui/dialog';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import PendingRequestRow from './PendingRequestRow.svelte';
  import { findDuplicatesByName } from '../utils/duplicateDevices';
  import { createCapabilitiesQuery, createDevicesQuery } from '../queries/deviceQueries';

  const queryClient = useQueryClient();
  const devicesQ = createDevicesQuery();
  const capabilitiesQ = createCapabilitiesQuery();

  let dismissed = $state<string[]>([]);

  const devices = $derived(devicesQ.data ?? []);
  const next = $derived(
    devices.find((d) => d.approval_type === 'pending' && !dismissed.includes(d.device_id))
  );
  const existing = $derived(
    next
      ? findDuplicatesByName(
          next,
          devices.filter(
            (d) => d.approved && d.approval_type !== 'revoked' && d.approval_type !== 'rejected'
          )
        )[0]
      : undefined
  );

  const refetch = () => queryClient.invalidateQueries({ queryKey: queryKeys.devices.all });

  let unlisten: UnlistenFn | undefined;
  listen('device://pending', refetch).then((fn) => (unlisten = fn));
  onDestroy(() => unlisten?.());
</script>

<Dialog.Root
  open={next !== undefined}
  onOpenChange={(o) => {
    if (!o && next) dismissed = [...dismissed, next.device_id];
  }}
>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>A phone wants to pair</Dialog.Title>
      <Dialog.Description>Allow it to connect to Portal Desktop?</Dialog.Description>
    </Dialog.Header>
    {#if next}
      {#key next.device_id}
        <PendingRequestRow
          bare
          device={next}
          capabilities={capabilitiesQ.data ?? []}
          {existing}
          onResolved={refetch}
        />
      {/key}
    {/if}
  </Dialog.Content>
</Dialog.Root>
