<!--
	Shown on the Devices page when LAN pairing is on but something looks likely
	to block other devices from reaching it: no Windows Firewall allow-rule
	found (`daemon::firewall`'s self-repair already tried and failed, most
	likely for lack of admin rights), and/or a connected VPN adapter (VPN
	clients commonly drop inbound LAN traffic as an anti-leak default — this
	is report-only, there's no "fix" an app can apply to someone else's VPN
	client). Dismiss is per-mount only — the underlying problem, not whether
	it's been seen, is what should decide whether this shows again.
-->
<script lang="ts">
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Button } from '$lib/components/ui/button';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';

  interface Props {
    firewallRuleMissing?: boolean;
    vpnAdapter?: string | null;
  }
  let { firewallRuleMissing = false, vpnAdapter = null }: Props = $props();

  let dismissed = $state(false);
</script>

{#if !dismissed}
  <Alert variant="destructive">
    <TriangleExclamation class="h-4 w-4" />
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div class="space-y-1">
        <AlertTitle
          >Other devices on this network may not be able to reach Portal Desktop</AlertTitle
        >
        <AlertDescription>
          <ul class="list-disc space-y-1 pl-4">
            {#if vpnAdapter}
              <li>
                A VPN looks active (<span class="font-medium">{vpnAdapter}</span>) — VPN clients
                often block incoming LAN connections. Try disconnecting it and pairing again.
              </li>
            {/if}
            {#if firewallRuleMissing}
              <li>
                No firewall rule found for Portal Desktop — check your firewall or antivirus
                software.
              </li>
            {/if}
          </ul>
        </AlertDescription>
      </div>
      <Button size="sm" variant="outline" class="shrink-0" onclick={() => (dismissed = true)}>
        Dismiss
      </Button>
    </div>
  </Alert>
{/if}
