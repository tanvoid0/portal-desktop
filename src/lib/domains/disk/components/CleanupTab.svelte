<script lang="ts">
  import { onMount } from 'svelte';
  import ScanProgressCard from './shared/ScanProgressCard.svelte';
  import TaskProgressCard from './shared/TaskProgressCard.svelte';
  import CleanupStats from './CleanupStats.svelte';
  import TargetPicker from './shared/TargetPicker.svelte';
  import SelectionBar from './shared/SelectionBar.svelte';
  import PermanentDeleteToggle from './shared/PermanentDeleteToggle.svelte';
  import CleanupSavedScanCard from './CleanupSavedScanCard.svelte';
  import CleanupVerifyCards from './CleanupVerifyCards.svelte';
  import CleanupProposalTable from './CleanupProposalTable.svelte';
  import { cleanupTab as st } from './cleanupTabState.svelte';
  import { Button } from '$lib/components/ui/button';

  let { pending }: { pending: { tab: string; path?: string; seq: number } | null } = $props();

  st.watch(() => pending);

  onMount(st.init);
</script>

<TargetPicker
  bind:value={st.root}
  locations={st.locations}
  busy={st.busy}
  placeholder="C:\Users\you\Downloads"
  showSuggestions={!st.summary}
  onscan={(p) => st.scan(p)}
  oncancel={st.cancelScan}
/>

{#if st.cached && !st.summary && !st.busy}
  <CleanupSavedScanCard
    cached={st.cached}
    onload={st.loadCached}
    onrescan={() => st.scan()}
    ondiscard={st.discardCached}
  />
{/if}

<ScanProgressCard progress={st.busy ? st.progress : null} />

<TaskProgressCard progress={st.qProgress} />

{#if st.status && !(st.busy && st.progress) && !st.qProgress}
  <div class="mb-4 text-sm text-muted-foreground">{st.status}</div>
{/if}

{#if st.summary}
  <CleanupStats proposals={st.summary.proposals} />

  <SelectionBar
    count={st.selected.size}
    bytes={st.selectedBytes}
    disabled={st.busy}
    onaction={st.quarantine}
    actionLabel={st.permanentDelete ? 'Remove selected' : 'Move to Recycle Bin'}
  >
    {#snippet tools()}
      <Button variant="outline" size="sm" onclick={st.selectSafe}>Select all Safe</Button>
      <Button variant="outline" size="sm" onclick={st.clearSelection}>Clear</Button>
      <PermanentDeleteToggle
        bind:checked={st.permanentDelete}
        id="cleanup-permanent"
        label="Delete Safe items permanently"
      />
      <Button variant="outline" size="sm" onclick={st.expandAll}>Expand all</Button>
      <Button variant="outline" size="sm" onclick={st.collapseAll}>Collapse all</Button>
      <Button variant="outline" size="sm" onclick={st.verifyWithAi} disabled={st.verifying}>
        {st.verifying
          ? 'Verifying…'
          : st.selected.size > 0
            ? `Ask AI about ${st.selected.size} selected`
            : 'Ask AI to double-check'}
      </Button>
      {#if st.verifying}
        <Button variant="outline" size="sm" onclick={st.cancelVerify}>Stop</Button>
      {/if}
    {/snippet}
  </SelectionBar>

  <CleanupVerifyCards
    verifying={st.verifying}
    progress={st.verifyProgress}
    verification={st.verification}
    error={st.verifyErr}
    oncancel={st.cancelVerify}
    ondismiss={st.dismissVerification}
    ondeselectFlagged={st.deselectFlagged}
  />

  <CleanupProposalTable
    rows={st.rows}
    selected={st.selected}
    expanded={st.expanded}
    verdictByPath={st.verdictByPath}
    empty={st.summary.proposals.length === 0}
    ontoggleSel={st.toggleSel}
    ontoggleExp={st.toggleExp}
  />
{/if}
