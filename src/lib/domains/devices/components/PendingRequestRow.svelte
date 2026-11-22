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
  import { bulkToggleable, withoutSensitive } from '../utils/capabilityChecklist';
  import { devicesUi } from '../state/devicesUi.svelte';
  import type { ApprovalType, CapabilityInfo, DeviceSummary } from '../types';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Check from '~icons/fa6-solid/check';
  import Xmark from '~icons/fa6-solid/xmark';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';

  interface Props {
    device: DeviceSummary;
    capabilities: CapabilityInfo[];
    existing?: DeviceSummary;
    onResolved?: () => void;
    /** Drop the card chrome — used when rendered inside the pair-request dialog. */
    bare?: boolean;
  }

  let { device, capabilities, existing, onResolved = () => {}, bare = false }: Props = $props();

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
  let approvalType = $state<ApprovalType>('long_term');
  let busy = $state(false);

  // Simple mode approves what the phone asked for, minus anything sensitive
  // (those need the advanced checklist), for 30 days. Nothing to read or pick.
  const simpleCaps = $derived.by(() => {
    const requested = withoutSensitive(device.capabilities, capabilities);
    return requested.length > 0 ? requested : bulkToggleable(capabilities);
  });
  const grantCaps = $derived(devicesUi.advanced ? selected : simpleCaps);
  const grantType = $derived<ApprovalType>(devicesUi.advanced ? approvalType : 'long_term');
  const sensitiveRequested = $derived(
    device.capabilities.filter((name) => !simpleCaps.includes(name))
  );

  // Advanced checklist starts from the same set, not an empty list.
  let seeded = false;
  $effect(() => {
    if (seeded || capabilities.length === 0) return;
    seeded = true;
    selected = simpleCaps;
  });

  // This request grants different access than the device's existing live
  // connection — most likely the app is re-asking for a wider scope or a
  // different lifetime rather than a plain reconnect, so the old grant is
  // probably stale.
  const existingDiffers = $derived.by(
    () =>
      existing !== undefined &&
      (existing.approval_type !== grantType || !sameCapabilities(existing.capabilities, grantCaps))
  );
  let revokeExisting = $state(true);

  async function allow() {
    busy = true;
    // Read before awaiting: the refetch after the approve drops this pending row
    // (and `existing` with it).
    const stale = existing && existingDiffers && revokeExisting ? existing.device_id : null;
    try {
      // Approve first: a failed approve must not leave the phone with no pairing at all.
      await approveDevice(device.device_id, grantType, grantCaps);
      if (stale) {
        await revokeDevice(stale).catch((err) =>
          toast.error('Approved, but the old pairing is still active', String(err))
        );
      }
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

<Card class={bare ? 'border-0 shadow-none' : 'border-primary/40'}>
  <CardContent class={bare ? 'space-y-2 p-0' : 'space-y-2 p-3'}>
    <div class="flex flex-wrap items-center gap-2">
      <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-medium" title={device.device_name}>{device.device_name}</p>
        <p class="truncate text-xs text-muted-foreground">
          asked {formatRelativeTime(new Date(device.created_at))} ·
          {grantCaps.length} capabilit{grantCaps.length === 1 ? 'y' : 'ies'} ·
          {APPROVAL_LABEL[grantType] ?? grantType}
        </p>
      </div>
      <div class="flex shrink-0 gap-2">
        <!-- No capabilities yet (loading, or the query failed): nothing safe to grant. -->
        <Button size="sm" onclick={allow} disabled={busy || capabilities.length === 0}>
          {#if busy}
            <Loader2 class="mr-1.5 h-3.5 w-3.5 animate-spin" />
          {:else}
            <Check class="mr-1.5 h-3.5 w-3.5" />
          {/if}
          Allow
        </Button>
        <Button size="sm" variant="outline" onclick={deny} disabled={busy}>
          <Xmark class="mr-1.5 h-3.5 w-3.5" />
          Deny
        </Button>
      </div>
    </div>

    {#if !devicesUi.advanced && sensitiveRequested.length > 0}
      <p class="text-xs text-muted-foreground">
        Not granting {sensitiveRequested.join(', ')} — turn on advanced mode to allow those.
      </p>
    {/if}

    {#if existing}
      <p class="text-xs text-muted-foreground">
        Already paired as this name · {APPROVAL_LABEL[existing.approval_type] ??
          existing.approval_type} · {existing.capabilities.join(', ') || 'no capabilities'}
      </p>
    {/if}

    {#if devicesUi.advanced}
      <div class="space-y-2 border-t pt-2">
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
      </div>
    {/if}
  </CardContent>
</Card>
