<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import * as RadioGroup from '$lib/components/ui/radio-group';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import CapabilityChecklist from './CapabilityChecklist.svelte';
  import { approveDevice, rejectDevice, revokeDevice } from '../api/deviceApi';
  import { toast } from '$lib/utils/toast';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { sameCapabilities } from '../utils/duplicateDevices';
  import type { ApprovalType, DeviceSummary } from '../types';
  import Loader2 from '~icons/fa6-solid/spinner';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';

  interface Props {
    device: DeviceSummary;
    capabilities: string[];
    existing?: DeviceSummary;
    onResolved?: () => void;
  }

  let { device, capabilities, existing, onResolved = () => {} }: Props = $props();

  const APPROVAL_OPTIONS: Array<{ value: ApprovalType; label: string }> = [
    { value: 'temporary', label: '24 hours' },
    { value: 'long_term', label: '30 days' },
    { value: 'until_revoked', label: 'Until revoked' },
  ];
  const APPROVAL_LABEL: Record<string, string> = {
    temporary: '24 hours',
    long_term: '30 days',
    until_revoked: 'Until revoked',
  };

  let selected = $state<string[]>([]);
  let approvalType = $state<ApprovalType>('temporary');
  let busy = $state(false);

  // This request grants different access than the device's existing live
  // connection — most likely the app is re-asking for a wider scope or a
  // different lifetime rather than a plain reconnect, so the old grant is
  // probably stale.
  const existingDiffers = $derived.by(
    () =>
      existing !== undefined &&
      (existing.approval_type !== approvalType ||
        !sameCapabilities(existing.capabilities, selected))
  );
  let revokeExisting = $state(true);

  async function allow() {
    busy = true;
    try {
      if (existing && existingDiffers && revokeExisting) {
        await revokeDevice(existing.device_id);
      }
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

    {#if existing}
      <p class="text-xs text-muted-foreground">
        Already paired as this name · {APPROVAL_LABEL[existing.approval_type] ??
          existing.approval_type} · {existing.capabilities.join(', ') || 'no capabilities'}
      </p>
    {/if}

    <CapabilityChecklist {capabilities} bind:selected disabled={busy} />

    <RadioGroup.Root bind:value={approvalType} class="flex flex-wrap gap-3" disabled={busy}>
      {#each APPROVAL_OPTIONS as option}
        <Label class="flex items-center gap-1.5 text-xs font-normal">
          <RadioGroup.Item value={option.value} />
          {option.label}
        </Label>
      {/each}
    </RadioGroup.Root>

    {#if existingDiffers}
      <Label class="flex items-start gap-1.5 text-xs font-normal text-destructive">
        <Checkbox bind:checked={revokeExisting} disabled={busy} class="mt-0.5" />
        <span class="flex items-center gap-1.5">
          <TriangleExclamation class="h-3 w-3 shrink-0" />
          This grants different access than its existing connection — archive the old one
        </span>
      </Label>
    {/if}

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
