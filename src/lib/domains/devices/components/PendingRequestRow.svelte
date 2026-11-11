<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import * as RadioGroup from '$lib/components/ui/radio-group';
  import { Label } from '$lib/components/ui/label';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { approveDevice, rejectDevice } from '../api/deviceApi';
  import { toast } from '$lib/utils/toast';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import type { ApprovalType, DeviceSummary } from '../types';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    device: DeviceSummary;
    capabilities: string[];
    onResolved?: () => void;
  }

  let { device, capabilities, onResolved = () => {} }: Props = $props();

  const APPROVAL_OPTIONS: Array<{ value: ApprovalType; label: string }> = [
    { value: 'temporary', label: '24 hours' },
    { value: 'long_term', label: '30 days' },
    { value: 'until_revoked', label: 'Until revoked' },
  ];

  let selected = $state<string[]>([]);
  let approvalType = $state<ApprovalType>('temporary');
  let busy = $state(false);

  async function allow() {
    busy = true;
    try {
      await approveDevice(device.device_id, approvalType, selected);
      toast.success('Device approved', { description: device.device_name });
      onResolved();
    } catch (err) {
      toast.error('Failed to approve device', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  }

  async function deny() {
    busy = true;
    try {
      await rejectDevice(device.device_id);
      onResolved();
    } catch (err) {
      toast.error('Failed to reject device', err instanceof Error ? err.message : String(err));
    } finally {
      busy = false;
    }
  }
</script>

<Card>
  <CardContent class="space-y-2 p-3">
    <div class="flex items-center justify-between gap-2">
      <p class="truncate text-sm font-medium" title={device.device_name}>{device.device_name}</p>
      <span class="shrink-0 text-xs text-muted-foreground">
        {formatRelativeTime(new Date(device.created_at))}
      </span>
    </div>

    <CapabilityChecklist {capabilities} bind:selected disabled={busy} />

    <RadioGroup.Root bind:value={approvalType} class="flex flex-wrap gap-3" disabled={busy}>
      {#each APPROVAL_OPTIONS as option}
        <Label class="flex items-center gap-1.5 text-xs font-normal">
          <RadioGroup.Item value={option.value} />
          {option.label}
        </Label>
      {/each}
    </RadioGroup.Root>

    <div class="flex gap-2">
      <Button size="sm" onclick={allow} disabled={busy} class="flex-1">
        {#if busy}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
        Allow
      </Button>
      <Button size="sm" variant="outline" onclick={deny} disabled={busy} class="flex-1">
        Deny
      </Button>
    </div>
  </CardContent>
</Card>
