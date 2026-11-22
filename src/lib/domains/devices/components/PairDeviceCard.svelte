<script lang="ts">
  import QRCode from 'qrcode';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { generatePairingQr, revokeDevice } from '../api/deviceApi';
  import { bulkToggleable } from '../utils/capabilityChecklist';
  import { devicesUi } from '../state/devicesUi.svelte';
  import { toast } from '$lib/utils/toast';
  import Loader2 from '~icons/fa6-solid/spinner';
  import QrCodeIcon from '~icons/fa6-solid/qrcode';
  import ArrowRotateRight from '~icons/fa6-solid/arrow-rotate-right';
  import Xmark from '~icons/fa6-solid/xmark';
  import type { CapabilityInfo } from '../types';

  interface Props {
    capabilities: CapabilityInfo[];
    /** device_ids currently in "Paired devices" — used to auto-clear the QR
     * once its grant has actually been scanned (see the effect below). */
    pairedDeviceIds?: Set<string>;
    onPaired?: () => void;
  }

  let { capabilities, pairedDeviceIds = new Set(), onPaired = () => {} }: Props = $props();

  let selected = $state<string[]>([]);
  let deviceId = $state<string | null>(null);
  let qrDataUrl = $state<string | null>(null);
  let qrUrl = $state('');
  let generating = $state(false);
  let canceling = $state(false);

  // Once the phone has actually scanned this QR, its grant shows up in
  // "Paired devices" — at that point the card has done its job, so drop it
  // instead of leaving a stale QR (and its now-dangerous Cancel button,
  // which would otherwise revoke a now-real paired device) on screen.
  $effect(() => {
    if (deviceId && pairedDeviceIds.has(deviceId)) {
      qrDataUrl = null;
      qrUrl = '';
      deviceId = null;
    }
  });

  // Scanning a QR completes pairing with no round trip through this window —
  // the backend has nothing to push an event on (see QA-110). Poll instead
  // while the code is on screen, so "Paired devices" picks up the scan
  // without a manual refresh. Matches the backend's 5-minute pickup window
  // (QR_PICKUP_TTL_MINUTES in pairing_service.rs); no point polling longer,
  // the token is dead by then.
  const QR_POLL_INTERVAL_MS = 3000;
  const QR_POLL_WINDOW_MS = 5 * 60 * 1000;

  $effect(() => {
    if (!qrDataUrl) return;
    const poll = setInterval(onPaired, QR_POLL_INTERVAL_MS);
    const stop = setTimeout(() => clearInterval(poll), QR_POLL_WINDOW_MS);
    return () => {
      clearInterval(poll);
      clearTimeout(stop);
    };
  });

  // Simple mode doesn't show the checklist, so it grants every non-sensitive
  // capability — enough for the phone to be useful, while terminal.exec and
  // friends stay behind the advanced checklist.
  const standardCaps = $derived(bulkToggleable(capabilities));
  const grantCaps = $derived(devicesUi.advanced ? selected : standardCaps);

  // Seed the advanced checklist with the same standard set simple mode grants,
  // so switching to advanced starts from "what you'd have got" rather than an
  // all-off list that generates a useless grant.
  let seeded = false;
  $effect(() => {
    if (seeded || capabilities.length === 0) return;
    seeded = true;
    selected = bulkToggleable(capabilities);
  });

  async function generate() {
    generating = true;
    try {
      const { qr_url, device_id } = await generatePairingQr(grantCaps);
      deviceId = device_id;
      qrUrl = qr_url;
      qrDataUrl = await QRCode.toDataURL(qr_url, { width: 240, margin: 2 });
      onPaired();
    } catch (err) {
      toast.error('Failed to generate QR code', err instanceof Error ? err.message : String(err));
    } finally {
      generating = false;
    }
  }

  async function cancel() {
    if (!deviceId) return;
    canceling = true;
    try {
      await revokeDevice(deviceId);
      qrDataUrl = null;
      qrUrl = '';
      deviceId = null;
      onPaired();
    } catch (err) {
      toast.error('Failed to cancel QR code', err instanceof Error ? err.message : String(err));
    } finally {
      canceling = false;
    }
  }
</script>

<Card>
  <CardContent class="space-y-3 p-4">
    {#if qrDataUrl}
      <div class="flex flex-wrap items-center gap-4">
        <img
          src={qrDataUrl}
          alt="Pairing QR code"
          class="h-36 w-36 shrink-0 rounded-md border bg-white p-1"
        />
        <div class="min-w-0 flex-1 space-y-2">
          <p class="text-sm font-medium">Scan this with the Portal app</p>
          <p class="text-xs text-muted-foreground">
            It connects straight away — no approval step on the phone. Expires in 5 minutes if
            nobody scans it.
          </p>
          <code class="block text-xs break-all text-muted-foreground">{qrUrl}</code>
        </div>
      </div>
    {:else}
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div class="min-w-0">
          <p class="text-sm font-medium">Pair a phone</p>
          <p class="text-xs text-muted-foreground">
            {#if devicesUi.advanced}
              Pick what the phone may do, then show it the code.
            {:else}
              Show the code to the phone — it gets the {standardCaps.length} standard capabilities.
            {/if}
          </p>
        </div>
        <Button onclick={generate} disabled={generating || canceling}>
          {#if generating}
            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          {:else}
            <QrCodeIcon class="mr-2 h-4 w-4" />
          {/if}
          Generate QR code
        </Button>
      </div>
    {/if}

    {#if devicesUi.advanced}
      <div class="border-t pt-3">
        <p class="mb-2 text-xs font-medium text-muted-foreground">Capabilities to grant</p>
        <CapabilityChecklist {capabilities} bind:selected disabled={generating} />
      </div>
    {/if}

    {#if qrDataUrl}
      <div class="flex gap-2 border-t pt-3">
        <Button variant="outline" size="sm" onclick={generate} disabled={generating || canceling}>
          {#if generating}
            <Loader2 class="mr-1.5 h-3.5 w-3.5 animate-spin" />
          {:else}
            <ArrowRotateRight class="mr-1.5 h-3.5 w-3.5" />
          {/if}
          Regenerate
        </Button>
        <Button variant="outline" size="sm" onclick={cancel} disabled={canceling || generating}>
          {#if canceling}
            <Loader2 class="mr-1.5 h-3.5 w-3.5 animate-spin" />
          {:else}
            <Xmark class="mr-1.5 h-3.5 w-3.5" />
          {/if}
          Cancel
        </Button>
      </div>
    {/if}
  </CardContent>
</Card>
