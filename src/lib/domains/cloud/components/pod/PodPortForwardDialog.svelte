<!-- Dialog to start a port forward to a pod -->
<script lang="ts">
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Button } from '$lib/components/ui/button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import { podApi } from '$lib/domains/cloud/api/podApi';
  import { toastActions } from '$lib/utils/toast';
  import type { PortForwardInfo } from '$lib/domains/cloud/providers/gcp/GCPTypes';

  interface Props {
    open: boolean;
    pod: ICloudResource;
    onStarted?: () => void;
  }
  let { open = $bindable(), pod, onStarted = () => {} }: Props = $props();

  let localPort = $state(8080);
  let remotePort = $state(80);
  let isPortForwarding = $state(false);
  let message = $state<string | null>(null);

  async function start() {
    try {
      isPortForwarding = true;
      message = null;
      const result = (await podApi.startPortForward({
        namespace: pod.namespace,
        podName: pod.name,
        localPort,
        remotePort,
      })) as PortForwardInfo;
      message = `Port forward started: ${result.url}`;
      toastActions.success(`Port forward started: ${result.url}`);
      open = false;
      onStarted();
    } catch (error) {
      message = error instanceof Error ? error.message : 'Failed to start port forward';
      toastActions.error(message);
    } finally {
      isPortForwarding = false;
    }
  }
</script>

<Dialog bind:open>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Port Forward</DialogTitle>
      <DialogDescription>Forward a local port to a port in pod {pod.name}</DialogDescription>
    </DialogHeader>
    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="local-port">Local Port</Label>
        <Input
          id="local-port"
          type="number"
          min="1"
          max="65535"
          bind:value={localPort}
          disabled={isPortForwarding}
        />
      </div>
      <div class="space-y-2">
        <Label for="remote-port">Remote Port</Label>
        <Input
          id="remote-port"
          type="number"
          min="1"
          max="65535"
          bind:value={remotePort}
          disabled={isPortForwarding}
        />
      </div>
      {#if message}
        <div class="rounded-md bg-muted p-3">
          <p class="text-sm">{message}</p>
        </div>
      {/if}
    </div>
    <DialogFooter>
      <Button
        variant="outline"
        onclick={() => {
          open = false;
          message = null;
        }}
        disabled={isPortForwarding}
      >
        Cancel
      </Button>
      <Button onclick={start} disabled={isPortForwarding}>
        {isPortForwarding ? 'Starting...' : 'Start Port Forward'}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
