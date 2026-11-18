<!-- Autonomy approval statistics card: per-action approval rates with refresh/retry. -->
<script lang="ts">
  import { Separator } from '$lib/components/ui/separator';
  import { Button } from '$lib/components/ui/button';
  import type { ApprovalStats } from '$lib/domains/autonomy';
  import TrendingUp from '~icons/fa6-solid/arrow-trend-up';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import SettingsSection from '../shared/SettingsSection.svelte';

  interface Props {
    stats: ApprovalStats | null;
    error: string | null;
    isRefreshing: boolean;
    onRefresh: () => void;
  }

  let { stats, error, isRefreshing, onRefresh }: Props = $props();

  const formatPercent = (num: number) => `${(num * 100).toFixed(1)}%`;
</script>

<SettingsSection
  title="Approval Statistics"
  description="Track autonomous action approval rates and patterns"
  icon={TrendingUp}
>
  {#if stats && Object.keys(stats).length > 0}
    <div class="space-y-3">
      {#each Object.entries(stats) as [actionType, s]}
        <div class="flex items-center justify-between rounded-md border p-3">
          <div class="flex-1">
            <div class="font-medium">{actionType}</div>
            <div class="text-sm text-muted-foreground">
              {s.approved} approved out of {s.total} requests
            </div>
          </div>
          <div class="text-right">
            <div class="text-lg font-bold">{formatPercent(s.approval_rate)}</div>
            <div class="text-xs text-muted-foreground">approval rate</div>
          </div>
        </div>
      {/each}
    </div>

    <Separator />

    <div class="flex justify-end">
      <Button variant="ghost" size="sm" onclick={onRefresh} disabled={isRefreshing}>
        <RefreshCw class="mr-2 h-4 w-4 {isRefreshing ? 'animate-spin' : ''}" />
        Refresh Stats
      </Button>
    </div>
  {:else if error}
    <div class="py-8 text-center text-sm text-destructive">
      <p>Failed to load approval statistics: {error}</p>
      <Button variant="ghost" size="sm" class="mt-2" onclick={onRefresh}>Retry</Button>
    </div>
  {:else}
    <div class="py-8 text-center text-muted-foreground">
      {#if isRefreshing}
        Loading statistics...
      {:else}
        <p>No approval statistics yet.</p>
        <p class="mt-2 text-xs">Statistics will appear here as autonomous actions are evaluated.</p>
      {/if}
    </div>
  {/if}
</SettingsSection>
