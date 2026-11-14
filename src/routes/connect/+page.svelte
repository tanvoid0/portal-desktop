<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Card from '$lib/components/ui/card';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { mobileConnection, type DiscoveredDesktop } from '$lib/domains/mobile/connection.svelte';
  import { parsePairUrl } from '$lib/domains/mobile/pairUrl';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { isBrowserClient } from '$lib/domains/mobile/browserShim';

  const DEFAULT_PORT = 18420;

  let desktops = $state<DiscoveredDesktop[]>([]);
  let searching = $state(true);
  let deviceName = $state(isBrowserClient ? 'This browser' : 'Android phone');
  // The daemon has no CORS layer, so a browser client's fetch can only ever reach the
  // origin the page itself was served from — prefill it rather than let someone type a
  // host that will just fail with an opaque "cannot reach" once the preflight is blocked.
  let host = $state(isBrowserClient ? window.location.hostname : '');
  let port = $state(
    isBrowserClient ? window.location.port || String(DEFAULT_PORT) : String(DEFAULT_PORT)
  );
  let error = $state<string | null>(null);
  /** Label of the desktop awaiting approval; null when idle. */
  let waitingFor = $state<string | null>(null);
  let connecting = $state(false);
  // Bumped on cancel so a late pair result from the abandoned attempt is ignored.
  let attempt = 0;

  const busy = $derived(waitingFor !== null || connecting);

  async function discover() {
    searching = true;
    try {
      desktops = await mobileConnection.discover();
    } catch {
      desktops = [];
    } finally {
      searching = false;
    }
  }

  onMount(discover);

  // Successful pair/connect navigates via the root layout gate (status event).
  async function pair(h: string, p: number, label: string) {
    const mine = ++attempt;
    error = null;
    waitingFor = label;
    try {
      await mobileConnection.pair(h, p, deviceName.trim() || 'Android phone');
    } catch (e) {
      if (mine === attempt) error = getErrorMessage(e, 'Connection failed');
    } finally {
      if (mine === attempt) waitingFor = null;
    }
  }

  function cancel() {
    attempt++;
    waitingFor = null;
  }

  function pairManual() {
    const p = Number(port);
    if (!host.trim() || !Number.isInteger(p) || p < 1 || p > 65535) {
      error = 'Enter a valid host and port.';
      return;
    }
    void pair(host.trim(), p, `${host.trim()}:${p}`);
  }

  async function scan() {
    error = null;
    try {
      const { scan: scanQr, Format } = await import('@tauri-apps/plugin-barcode-scanner');
      const res = await scanQr({ formats: [Format.QRCode] });
      const parsed = parsePairUrl(res.content);
      if (!parsed) {
        error = 'That QR code is not a Portal pairing code.';
        return;
      }
      connecting = true;
      await mobileConnection.connect(parsed.host, parsed.port, parsed.token);
    } catch (e) {
      error = getErrorMessage(e, 'Connection failed');
    } finally {
      connecting = false;
    }
  }
</script>

<svelte:head><title>Connect - Portal Mobile</title></svelte:head>

<div class="mx-auto flex w-full max-w-md flex-col gap-4 p-4">
  <div>
    <h1 class="text-xl font-semibold">Connect to your PC</h1>
    <p class="text-sm text-muted-foreground">Portal Desktop must be running on the same network.</p>
  </div>

  {#if error}
    <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert>
  {/if}

  {#if waitingFor}
    <Card.Root>
      <Card.Header>
        <Card.Title>Approve on your PC…</Card.Title>
        <Card.Description>
          Click Allow on the pairing request from {deviceName || 'this phone'} on {waitingFor}.
        </Card.Description>
      </Card.Header>
      <Card.Footer>
        <Button variant="outline" class="w-full" onclick={cancel}>Cancel</Button>
      </Card.Footer>
    </Card.Root>
  {:else}
    <div class="grid gap-2">
      <Label for="device-name">This phone's name</Label>
      <Input id="device-name" bind:value={deviceName} disabled={busy} />
    </div>

    <Button size="lg" class="w-full" onclick={scan} disabled={busy}>
      {connecting ? 'Connecting…' : 'Scan QR code'}
    </Button>

    <Card.Root>
      <Card.Header>
        <Card.Title>Desktops on this network</Card.Title>
      </Card.Header>
      <Card.Content class="grid gap-2">
        {#each desktops as d (d.host + d.port)}
          <Button
            variant="outline"
            class="h-auto justify-start py-3"
            disabled={busy}
            onclick={() => pair(d.host, d.port, d.name)}
          >
            <span class="flex flex-col items-start">
              <span class="font-medium">{d.name}</span>
              <span class="text-xs text-muted-foreground">{d.host}:{d.port}</span>
            </span>
          </Button>
        {:else}
          <p class="text-sm text-muted-foreground">
            {searching ? 'Searching…' : 'No desktops found.'}
          </p>
        {/each}
        <Button variant="ghost" size="sm" onclick={discover} disabled={searching || busy}>
          Search again
        </Button>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Enter address</Card.Title>
      </Card.Header>
      <Card.Content class="grid gap-3">
        <div class="grid gap-2">
          <Label for="host">IP address</Label>
          <Input
            id="host"
            bind:value={host}
            placeholder="192.168.1.20"
            inputmode="decimal"
            autocapitalize="none"
            disabled={busy}
          />
        </div>
        <div class="grid gap-2">
          <Label for="port">Port</Label>
          <Input id="port" bind:value={port} inputmode="numeric" disabled={busy} />
        </div>
      </Card.Content>
      <Card.Footer>
        <Button class="w-full" onclick={pairManual} disabled={busy}>Pair</Button>
      </Card.Footer>
    </Card.Root>
  {/if}
</div>
