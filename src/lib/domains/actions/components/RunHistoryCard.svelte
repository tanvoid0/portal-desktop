<!-- Project-scoped local run history with refresh. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { PageEmpty, PageLoading } from '$lib/components/shell';
  import History from '~icons/fa6-solid/clock-rotate-left';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { RunSummary } from '../types';
  import ActionRunList from './ActionRunList.svelte';

  interface Props {
    runs: RunSummary[];
    loading: boolean;
    selectedId: string | null;
    onRefresh: () => void;
    onSelect: (run: RunSummary, opts?: { terminal?: boolean }) => void;
    onStarted: (id: string) => void;
    onChanged: (deletedId?: string) => void;
  }

  let { runs, loading, selectedId, onRefresh, onSelect, onStarted, onChanged }: Props = $props();
</script>

<SectionCard
  title="Local run history"
  description="Script executions for this project (Actions runs)"
  icon={History}
  titleClass="text-base"
>
  {#snippet actions()}
    <Button variant="ghost" size="sm" onclick={onRefresh} disabled={loading}>
      <RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
    </Button>
  {/snippet}
  {#if loading && runs.length === 0}
    <PageLoading message="Loading history…" />
  {:else if runs.length === 0}
    <PageEmpty
      title="No action runs yet"
      description="Run an action to see command output and exit details here."
      icon={History}
    />
  {:else}
    <ActionRunList
      executions={runs}
      {selectedId}
      onSelect={(e, opts) => onSelect(e, opts)}
      {onStarted}
      {onChanged}
    />
  {/if}
</SectionCard>
