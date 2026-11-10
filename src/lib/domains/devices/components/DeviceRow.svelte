<script lang="ts">
  import * as Table from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { revokeDevice, updateDeviceCapabilities } from '../api/deviceApi';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import type { DeviceSummary } from '../types';
  import Pencil from '~icons/fa6-solid/pen';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    device: DeviceSummary;
    capabilities: string[];
    onChanged?: () => void;
  }

  let { device, capabilities, onChanged = () => {} }: Props = $props();

  const APPROVAL_LABEL: Record<string, string> = {
    pending: 'Pending',
    temporary: 'Temporary (24h)',
    long_term: 'Long-term (30d)',
    until_revoked: 'Until revoked',
    revoked: 'Revoked',
    rejected: 'Rejected',
  };

  let editing = $state(false);
  let selected = $state<string[]>([...device.capabilities]);
  let busy = $state(false);

  const isActive = $derived(device.approved && device.approval_type !== 'revoked');

  async function saveCapabilities() {
    busy = true;
    try {
      await updateDeviceCapabilities(device.device_id, selected);
      toast.success('Capabilities updated');
      editing = false;
      onChanged();
    } catch (err) {
      toast.error(
        'Failed to update capabilities',
        err instanceof Error ? err.message : String(err)
      );
    } finally {
      busy = false;
    }
  }

  async function revoke() {
    const confirmed = await confirmAction(
      `Revoke access for ${device.device_name}? It will have to be paired again.`,
      'Revoke device'
    );
    if (!confirmed) return;

    busy = true;
    try {
      await revokeDevice(device.device_id);
      toast.success('Device revoked');
      onChanged();
    } catch (err) {
      toast.error('Failed to revoke device', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  }
</script>

<Table.Row>
  <Table.Cell class="font-medium">{device.device_name}</Table.Cell>
  <Table.Cell>
    <Badge variant={isActive ? 'default' : 'outline'}>
      {APPROVAL_LABEL[device.approval_type] ?? device.approval_type}
    </Badge>
  </Table.Cell>
  <Table.Cell>
    {#if editing}
      <div class="space-y-2">
        <CapabilityChecklist {capabilities} bind:selected disabled={busy} />
        <div class="flex gap-2">
          <Button size="sm" onclick={saveCapabilities} disabled={busy}>Save</Button>
          <Button size="sm" variant="outline" onclick={() => (editing = false)} disabled={busy}>
            Cancel
          </Button>
        </div>
      </div>
    {:else}
      <div class="flex flex-wrap items-center gap-1">
        {#each device.capabilities as cap}
          <Badge variant="secondary" class="text-xs">{cap}</Badge>
        {:else}
          <span class="text-xs text-muted-foreground">none</span>
        {/each}
        {#if isActive}
          <Button
            variant="ghost"
            size="icon"
            class="h-6 w-6"
            onclick={() => {
              selected = [...device.capabilities];
              editing = true;
            }}
            aria-label="Edit capabilities"
          >
            <Pencil class="h-3 w-3" />
          </Button>
        {/if}
      </div>
    {/if}
  </Table.Cell>
  <Table.Cell class="text-xs text-muted-foreground">
    {device.last_used_at ? new Date(device.last_used_at).toLocaleString() : 'Never'}
  </Table.Cell>
  <Table.Cell class="text-xs text-muted-foreground">
    {device.expires_at ? new Date(device.expires_at).toLocaleString() : '—'}
  </Table.Cell>
  <Table.Cell>
    {#if isActive}
      <Button variant="destructive" size="sm" onclick={revoke} disabled={busy}>
        {#if busy}
          <Loader2 class="h-4 w-4 animate-spin" />
        {:else}
          Revoke
        {/if}
      </Button>
    {/if}
  </Table.Cell>
</Table.Row>
