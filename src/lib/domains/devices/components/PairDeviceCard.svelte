<script lang="ts">
  import QRCode from 'qrcode';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { generatePairingQr } from '../api/deviceApi';
  import { toast } from '$lib/utils/toast';
  import Loader2 from '~icons/fa6-solid/spinner';
  import QrCodeIcon from '~icons/fa6-solid/qrcode';

  interface Props {
    capabilities: string[];
    onPaired?: () => void;
  }

  let { capabilities, onPaired = () => {} }: Props = $props();

  let selected = $state<string[]>([]);
  let qrDataUrl = $state<string | null>(null);
  let qrUrl = $state('');
  let generating = $state(false);

  async function generate() {
    generating = true;
    try {
      const { qr_url } = await generatePairingQr(selected);
      qrUrl = qr_url;
      qrDataUrl = await QRCode.toDataURL(qr_url, { width: 240, margin: 2 });
      onPaired();
    } catch (err) {
      toast.error('Failed to generate QR code', err instanceof Error ? err.message : String(err));
    } finally {
      generating = false;
    }
  }
</script>

<Card>
  <CardHeader>
    <CardTitle>Pair a phone</CardTitle>
  </CardHeader>
  <CardContent class="space-y-4">
    <div>
      <p class="mb-2 text-sm font-medium">Capabilities to grant</p>
      <CapabilityChecklist {capabilities} bind:selected disabled={generating} />
    </div>

    {#if qrDataUrl}
      <div class="flex flex-col items-center gap-3 rounded-lg border bg-white p-4">
        <img src={qrDataUrl} alt="Pairing QR code" class="h-56 w-56" />
        <code class="text-center text-xs break-all text-muted-foreground">{qrUrl}</code>
      </div>
      <p class="text-xs text-muted-foreground">
        Scanning connects immediately with the capabilities above — there's no approval step on the
        phone. The grant appears below once scanned; an unscanned code expires in 5 minutes.
      </p>
    {/if}

    <Button onclick={generate} disabled={generating}>
      {#if generating}
        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
      {:else}
        <QrCodeIcon class="mr-2 h-4 w-4" />
      {/if}
      {qrDataUrl ? 'Regenerate QR code' : 'Generate QR code'}
    </Button>
  </CardContent>
</Card>
