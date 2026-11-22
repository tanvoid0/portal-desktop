<script lang="ts">
  import QRCode from 'qrcode';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { generatePairingQr } from '../api/deviceApi';
  import { toast } from '$lib/utils/toast';
  import type { DeviceSummary } from '../types';
  import Loader2 from '~icons/fa6-solid/spinner';
  import QrCodeIcon from '~icons/fa6-solid/qrcode';

  interface Props {
    device: DeviceSummary;
    onChanged?: () => void;
  }

  let { device, onChanged = () => {} }: Props = $props();

  // A grant with no capabilities reconnects the phone to nothing, so there's
  // no point minting a code for it — and silently widening the set here would
  // hand out access the user never granted.
  const nothingToGrant = $derived(device.capabilities.length === 0);

  let open = $state(false);
  let qrDataUrl = $state<string | null>(null);
  let qrUrl = $state('');
  let busy = $state(false);

  // Mints a *fresh* grant carrying this device's current capabilities — there's
  // no command to re-issue a token for an existing row, and a phone that lost
  // its credential needs a new one anyway. Scanning it leaves the old grant in
  // place, which the duplicate-name warning then offers to archive.
  async function show() {
    busy = true;
    try {
      const { qr_url } = await generatePairingQr(device.capabilities);
      qrUrl = qr_url;
      qrDataUrl = await QRCode.toDataURL(qr_url, { width: 240, margin: 2 });
      open = true;
      onChanged();
    } catch (err) {
      toast.error('Failed to generate QR code', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  }
</script>

<Button
  variant="ghost"
  size="icon"
  class="h-7 w-7"
  onclick={show}
  disabled={busy || nothingToGrant}
  title={nothingToGrant
    ? 'No capabilities on this grant — edit them before reconnecting'
    : 'Reconnect with a QR code'}
  aria-label="Reconnect {device.device_name} with a QR code"
>
  {#if busy}
    <Loader2 class="h-3.5 w-3.5 animate-spin" />
  {:else}
    <QrCodeIcon class="h-3.5 w-3.5" />
  {/if}
</Button>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Reconnect {device.device_name}</Dialog.Title>
      <Dialog.Description>
        Same capabilities as now ({device.capabilities.length}) — scan and it's connected. Expires
        in 5 minutes if nobody scans it.
      </Dialog.Description>
    </Dialog.Header>
    {#if qrDataUrl}
      <div class="flex flex-col items-center gap-2">
        <img
          src={qrDataUrl}
          alt="Pairing QR code"
          class="h-48 w-48 rounded-md border bg-white p-1"
        />
        <code class="text-center text-xs break-all text-muted-foreground">{qrUrl}</code>
      </div>
    {/if}
  </Dialog.Content>
</Dialog.Root>
