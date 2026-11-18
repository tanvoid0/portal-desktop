<!-- Learning memory usage card: stats, pending cleanup and refresh. -->
<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import { Separator } from '$lib/components/ui/separator';
  import { Button } from '$lib/components/ui/button';
  import type { MemoryStats, CleanupPreview } from '$lib/domains/learning';
  import Database from '~icons/fa6-solid/database';
  import Trash2 from '~icons/fa6-solid/trash';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import SettingsSection from '../shared/SettingsSection.svelte';
  import { formatNumber } from './format';

  interface Props {
    memoryStats: MemoryStats | null;
    cleanupPreview: CleanupPreview | null;
    isLoading: boolean;
    isRefreshing: boolean;
    onCleanup: () => void;
    onRefresh: () => void;
  }

  let { memoryStats, cleanupPreview, isLoading, isRefreshing, onCleanup, onRefresh }: Props =
    $props();
</script>

<SettingsSection
  title="Memory Usage"
  description="Storage and retention statistics"
  icon={Database}
>
  {#if memoryStats}
    <div class="grid grid-cols-3 gap-4">
      <div>
        <Label class="text-sm text-muted-foreground">Learning Events</Label>
        <div class="mt-1 text-2xl font-bold">
          {formatNumber(memoryStats.total_events)}
        </div>
        <div class="text-xs text-muted-foreground">
          Retention: {memoryStats.events_retention_days} days
        </div>
      </div>
      <div>
        <Label class="text-sm text-muted-foreground">Patterns Learned</Label>
        <div class="mt-1 text-2xl font-bold">
          {formatNumber(memoryStats.total_patterns)}
        </div>
        <div class="text-xs text-muted-foreground">
          Max: {formatNumber(memoryStats.max_patterns_per_type)} per type
        </div>
      </div>
      <div>
        <Label class="text-sm text-muted-foreground">Preferences</Label>
        <div class="mt-1 text-2xl font-bold">
          {formatNumber(memoryStats.total_preferences)}
        </div>
        <div class="text-xs text-muted-foreground">User preferences stored</div>
      </div>
    </div>

    {#if cleanupPreview}
      <Separator />
      <div class="space-y-2">
        <Label class="text-sm">Pending Cleanup</Label>
        <div class="flex items-center justify-between text-sm">
          <span class="text-muted-foreground">
            {cleanupPreview.events_to_delete} events, {cleanupPreview.patterns_to_delete}
            patterns
          </span>
          <Button
            variant="outline"
            size="sm"
            onclick={onCleanup}
            disabled={isRefreshing ||
              (cleanupPreview.events_to_delete === 0 && cleanupPreview.patterns_to_delete === 0)}
          >
            <Trash2 class="mr-2 h-4 w-4" />
            Clean Up
          </Button>
        </div>
      </div>
    {/if}

    <Separator />

    <div class="flex justify-end">
      <Button variant="ghost" size="sm" onclick={onRefresh} disabled={isRefreshing}>
        <RefreshCw class="mr-2 h-4 w-4 {isRefreshing ? 'animate-spin' : ''}" />
        Refresh Stats
      </Button>
    </div>
  {:else}
    <div class="py-8 text-center text-muted-foreground">
      {#if isLoading}
        Loading statistics...
      {:else}
        Failed to load statistics
      {/if}
    </div>
  {/if}
</SettingsSection>
