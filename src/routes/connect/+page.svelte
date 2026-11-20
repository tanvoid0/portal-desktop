<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Card from '$lib/components/ui/card';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Badge } from '$lib/components/ui/badge';
  import { mobileConnection, type DiscoveredDesktop } from '$lib/domains/mobile/connection.svelte';
  import { parsePairUrl } from '$lib/domains/mobile/pairUrl';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { isBrowserClient } from '$lib/domains/mobile/browserShim';

  const DEFAULT_PORT = 18420;
  // How long discovery keeps re-probing before it gives up and shows the empty state,
  // and how often it re-broadcasts while doing so — matches portal_remote's PairScreen.
  const SEARCH_WINDOW_MS = 10_000;
  const PROBE_INTERVAL_MS = 800;

  let desktops = $state<DiscoveredDesktop[]>([]);
  let searching = $state(true);
  // True once the 10s window above has elapsed and nothing was found.
  let searchTimedOut = $state(false);
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
  // Discovery came up empty and a connect attempt (manual entry or QR) also failed —
  // on a phone a VPN is the usual cause: it routes traffic off the LAN entirely, so
  // nothing here was ever reachable to begin with.
  const vpnLikely = $derived(searchTimedOut && error !== null);
  // The previously-paired PC (by stable id, survives its IP changing) sorts first.
  const sortedDesktops = $derived.by(() =>
    [...desktops].sort((a, b) => Number(b.paired) - Number(a.paired))
  );

  // Bumped so a stale discover() loop (unmounted, or superseded by a fresh "Search
  // again") stops touching state and stops re-probing.
  let discoverGen = 0;

  async function discover() {
    const gen = ++discoverGen;
    searching = true;
    searchTimedOut = false;
    desktops = [];
    const found = new Map<string, DiscoveredDesktop>();
    const probe = async () => {
      try {
        for (const d of await mobileConnection.discover()) found.set(`${d.host}:${d.port}`, d);
        if (gen === discoverGen) desktops = [...found.values()];
      } catch {
        // one failed broadcast doesn't end the search — the deadline below does
      }
    };
    const deadline = Date.now() + SEARCH_WINDOW_MS;
    await probe();
    while (gen === discoverGen && Date.now() < deadline) {
      await new Promise((r) => setTimeout(r, PROBE_INTERVAL_MS));
      if (gen === discoverGen) void probe();
    }
    if (gen === discoverGen) {
      searching = false;
      searchTimedOut = found.size === 0;
    }
  }

  onMount(() => {
    void discover();
    return () => discoverGen++; // leaving the page cancels the loop above
  });

  // Successful pair/connect navigates via the root layout gate (status event).
  async function pair(h: string, p: number, label: string, id: string | null = null) {
    const mine = ++attempt;
    error = null;
    waitingFor = label;
    try {
      await mobileConnection.pair(h, p, deviceName.trim() || 'Android phone', id);
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

  {#if vpnLikely}
    <Alert>
      <AlertDescription>
        Still nothing? A VPN on this phone can route traffic away from your local network, so a PC
        on the same Wi-Fi becomes unreachable. Try turning it off, or allow local network access in
        its settings.
      </AlertDescription>
    </Alert>
  {/if}

  {#if waitingFor}
    <Card.Root>
      <Card.Header class="items-center text-center">
        <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
        <Card.Title>Waiting for {waitingFor}</Card.Title>
        <Card.Description>
          Approve the request on {waitingFor}'s screen — it's asking there as {deviceName ||
            'this phone'}.
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
        {#each sortedDesktops as d (d.host + d.port)}
          <Button
            variant="outline"
            class="h-auto justify-start py-3"
            disabled={busy}
            onclick={() => pair(d.host, d.port, d.name, d.id)}
          >
            <span class="flex flex-1 flex-col items-start">
              <span class="font-medium">{d.name}</span>
              <span class="text-xs text-muted-foreground">{d.host}:{d.port}</span>
            </span>
            {#if d.paired}
              <Badge variant="secondary">Previously paired</Badge>
            {/if}
          </Button>
        {:else}
          <p class="text-sm text-muted-foreground">
            {#if searching}
              Searching…
            {:else if searchTimedOut}
              No desktops found after 10s. Make sure your phone is on the same Wi-Fi as your PC.
            {:else}
              No desktops found.
            {/if}
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
