<!--
	Project Audit — bulk discovery and triage.

	The registry answers "what am I tracking"; this answers "what is actually on
	disk, is it alive, and is any of its work only here". Scanning writes into the
	same projects table, so a project triaged here is the same row the rest of the
	app already uses.
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import * as Tabs from '$lib/components/ui/tabs';
  import { PageStats, ResponsivePanel } from '$lib/components/shell';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import Download from '~icons/fa6-solid/download';
  import { cancelDiscovery } from '$lib/domains/projects/api/discoveryApi';
  import {
    auditRowId as rowId,
    ProjectAuditState,
    type AuditTab as Tab,
  } from '$lib/domains/projects/state/projectAudit.svelte';
  import AuditProjectCard from './AuditProjectCard.svelte';
  import AuditChangesList from './AuditChangesList.svelte';
  import AuditDuplicateSets from './AuditDuplicateSets.svelte';
  import AuditTree from './AuditTree.svelte';
  import AuditScanPanel from './AuditScanPanel.svelte';
  import AuditFocusedDetail from './AuditFocusedDetail.svelte';
  import AuditMarkedActions from './AuditMarkedActions.svelte';

  const a = new ProjectAuditState();
</script>

<svelte:window onkeydown={a.onKeydown} />

<!--
	The configuration used to sit on top of the results: at 1280x800 the first
	triage row landed at y=676 of a 760px viewport and at 1024x600 none was on
	screen at all (UX-009). Scan config, the cursor row's facts and the bulk
	actions now live in a right rail, which ResponsivePanel folds into a Sheet
	below `md`.
-->
<div class="flex items-start gap-4">
  <div class="min-w-0 flex-1 space-y-3">
    {#if a.stats}
      <PageStats stats={a.statStrip} />
    {/if}

    <!-- Once the rail is a Sheet, this button is the only way into it. -->
    <div class="flex items-center gap-2 md:hidden">
      <Button variant="outline" size="sm" onclick={() => (a.railOpen = true)}>
        <RefreshCw class="mr-1 h-3 w-3" /> Scan and details
      </Button>
      {#if a.marked.size}
        <Badge variant="secondary">{a.marked.size} selected</Badge>
      {/if}
    </div>

    {#if !a.report && !a.loadingState}
      <p class="py-8 text-center text-sm text-muted-foreground">
        Nothing scanned yet — add a folder and press Scan and import.
      </p>
    {/if}

    {#if a.report}
      {#snippet cardList()}
        <div class="space-y-2">
          {#each a.filtered as p, i (p.path)}
            <AuditProjectCard
              project={p}
              id={rowId(p.path)}
              isCursor={i === a.cursor}
              isSelected={a.selected?.path === p.path}
              marked={a.marked.has(p.path)}
              decision={a.decisions[p.path] ?? null}
              onToggleMark={() => a.toggleMark(p.path)}
              onSelect={() => {
                a.cursor = i;
                a.selected = a.selected?.path === p.path ? null : p;
              }}
              onDecide={(d) => a.decide(p, d)}
              onOpen={a.openFolder}
              onClean={() => a.cleanInDisk(p)}
            />
          {:else}
            <p class="py-8 text-center text-sm text-muted-foreground">Nothing matches.</p>
          {/each}
        </div>
      {/snippet}

      <Tabs.Root value={a.tab} onValueChange={(v) => (a.tab = v as Tab)} class="gap-4">
        <div class="flex flex-wrap items-center gap-2">
          <Tabs.List>
            <Tabs.Trigger value="triage">Triage</Tabs.Trigger>
            <Tabs.Trigger value="risk">At risk</Tabs.Trigger>
            <Tabs.Trigger value="duplicates">Duplicates</Tabs.Trigger>
            <Tabs.Trigger value="tree">Tree</Tabs.Trigger>
            <Tabs.Trigger value="changes">
              Changes{a.changes.length ? ` (${a.changes.length})` : ''}
            </Tabs.Trigger>
          </Tabs.List>
          <Button
            size="sm"
            variant={a.sortBySize ? 'default' : 'outline'}
            onclick={() => (a.sortBySize = !a.sortBySize)}
            title="Order by reclaimable build output instead of last activity"
          >
            <HardDrive class="mr-1 h-3 w-3" /> By size
          </Button>
          <Button
            size="sm"
            variant="outline"
            onclick={a.exportCsv}
            title="Export the current list as CSV"
          >
            <Download class="mr-1 h-3 w-3" /> Export
          </Button>
          <Input
            bind:value={a.search}
            aria-label="Search the audit list"
            placeholder="Search…"
            class="ml-auto max-w-xs"
          />
        </div>

        {#if a.tab === 'triage' || a.tab === 'risk'}
          <p class="text-xs text-muted-foreground">
            <span class="font-mono">j</span>/<span class="font-mono">k</span> move ·
            <span class="font-mono">1</span>–<span class="font-mono">4</span>
            keep/archive/delete/review · <span class="font-mono">o</span> open folder ·
            <span class="font-mono">x</span>
            select ·
            <span class="font-mono">esc</span> clear
          </p>
        {/if}

        {#if a.exportedTo}
          <p class="font-mono text-xs text-muted-foreground">Written to {a.exportedTo}</p>
        {/if}

        <Tabs.Content value="changes"><AuditChangesList changes={a.changes} /></Tabs.Content>

        <Tabs.Content value="duplicates">
          <AuditDuplicateSets sets={a.duplicateSets} onInspect={(p) => (a.selected = p)} />
        </Tabs.Content>

        <Tabs.Content value="tree">
          <AuditTree
            projects={a.filtered}
            searching={!!a.search.trim()}
            onInspect={(p) => (a.selected = p)}
          />
        </Tabs.Content>

        <!-- Same list, different filter — `filtered` narrows it by the active tab. -->
        <Tabs.Content value="triage">{@render cardList()}</Tabs.Content>
        <Tabs.Content value="risk">{@render cardList()}</Tabs.Content>
      </Tabs.Root>
    {/if}
  </div>

  <!--
  	Read-only except for the scan controls, so the keyboard loop is untouched:
  	`onKeydown` already bails when the event target is an input, which covers
  	the path field in here exactly as it covers the search box.
  -->
  <ResponsivePanel
    bind:open={a.railOpen}
    side="right"
    title="Scan and triage"
    desktopClass="sticky top-0 max-h-[calc(100vh-7rem)] w-80 self-start rounded-lg border bg-card"
    borderClass=""
  >
    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-3">
      <AuditScanPanel
        roots={a.roots}
        scanning={a.scanning}
        loadingState={a.loadingState}
        progress={a.progress}
        error={a.error}
        importSummary={a.importSummary}
        bind:scanOpen={a.scanOpen}
        bind:newRootPath={a.newRootPath}
        bind:measureSizes={a.measureSizes}
        bind:markMissing={a.markMissing}
        onScan={a.scan}
        onCancel={() => cancelDiscovery()}
        onAddRoot={a.addRoot}
        onRemoveRoot={a.removeRoot}
        onPersist={a.persistConfig}
      />
      <AuditFocusedDetail project={a.focused} siblings={a.duplicateSiblings} />
      <AuditMarkedActions count={a.marked.size} onDecide={a.decideMarked} onClear={a.clearMarked} />
    </div>
  </ResponsivePanel>
</div>
