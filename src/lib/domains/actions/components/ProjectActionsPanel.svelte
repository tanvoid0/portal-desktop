<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import Play from '~icons/fa6-solid/play';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import type { Project } from '$lib/domains/projects/types';
  import { GitHubProjectActionsPanel } from '$lib/domains/github';
  import ReleaseCard from './ReleaseCard.svelte';
  import ActionRunDetail from './ActionRunDetail.svelte';
  import ActionCatalogSidebar from './ActionCatalogSidebar.svelte';
  import PipelineGeneratorCard from './PipelineGeneratorCard.svelte';
  import DoctorFailureCard from './DoctorFailureCard.svelte';
  import DoctorProposalCard from './DoctorProposalCard.svelte';
  import DoctorReportCard from './DoctorReportCard.svelte';
  import LastRunCard from './LastRunCard.svelte';
  import RunHistoryCard from './RunHistoryCard.svelte';
  import PipelineRunDetail from '$lib/domains/projects/pipelines/components/PipelineRunDetail.svelte';
  import { dismissReviewPointer } from '../state/doctor.svelte';
  import { ProjectActionsState } from '../state/projectActions.svelte';

  interface Props {
    project: Project;
    enabled?: boolean;
  }

  let { project, enabled = true }: Props = $props();

  const a = new ProjectActionsState(() => ({ project, enabled }));

  const SUBS = ['catalog', 'history', 'generate', 'release', 'github'];
  const initialSub = page.url.searchParams.get('sub');
  let tab = $state(SUBS.includes(initialSub ?? '') ? (initialSub as string) : 'catalog');
  const failedRuns = $derived(a.runHistory.filter((r) => r.status === 'failed').length);

  // Keep the sub-tab in the URL (replace, no history entry) so refresh and links land on it.
  $effect(() => {
    const params = new URLSearchParams(page.url.search);
    if (params.get('sub') === tab) return;
    params.set('sub', tab);
    void goto(`${page.url.pathname}?${params}`, { replaceState: true, reset: false });
  });
  const showGithub = $derived(a.sourceFilter === 'all' || a.sourceFilter === 'github');

  // Opening a run (from anywhere) jumps to the History tab, where its output lives.
  $effect(() => {
    if (a.selectedRunId) tab = 'history';
  });
  // Starting a run with no selection yet still shows its result in History.
  $effect(() => {
    if (a.lastResult) tab = 'history';
  });
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div>
      <h2 class="text-xl font-semibold">Actions</h2>
      <p class="text-sm text-muted-foreground">
        Smart defaults from project metadata
        {#if a.filePath}
          · overrides from <code class="text-xs">{a.filePath}</code>
        {/if}
        · GitHub & n8n when available
      </p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button
        variant="outline"
        size="sm"
        class="gap-2"
        onclick={() => {
          void a.loadCatalog();
          void a.loadHistory();
        }}
        disabled={a.loading || a.running}
      >
        <RefreshCw class="h-4 w-4" />
        Refresh
      </Button>
      {#if a.selected.size > 0 || a.running}
        <Button
          type="button"
          size="sm"
          class="gap-2"
          onclick={() => void a.runSelected()}
          disabled={a.running}
        >
          <Play class="h-4 w-4" />
          {a.running ? 'Running…' : `Run selected (${a.selected.size})`}
        </Button>
      {/if}
    </div>
  </div>

  {#if a.shownFailure}
    {@const failure = a.shownFailure}
    <DoctorFailureCard
      {failure}
      run={a.doctorRun}
      busy={a.doctorBusy || a.genRunning}
      onDiagnose={() => void a.diagnoseFailure(failure)}
      onOpenThread={(id) => void a.openThread(id)}
    />
  {/if}

  {#if a.doctorRun.proposal}
    <DoctorProposalCard
      proposal={a.doctorRun.proposal}
      reviewThreadId={a.doctorRun.reviewThreadId}
      onVerify={() => void a.verifyFix()}
      onOpenThread={(id) => void a.openThread(id)}
    />
  {/if}

  {#if a.doctorRun.reviewThreadId && !a.shownFailure && !a.doctorRun.proposal}
    <Card class="border-primary/30">
      <CardContent class="flex flex-wrap items-center gap-2 py-3">
        <span class="text-sm">
          {a.doctorRun.status === 'fixed' ? 'Fixed' : 'Agent changes'} · pending your review in Coder
        </span>
        <Button
          type="button"
          size="sm"
          variant="link"
          onclick={() => void a.openThread(a.doctorRun.reviewThreadId!)}
        >
          Review changes in Coder
        </Button>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          class="ml-auto"
          onclick={() => dismissReviewPointer(project.id)}
        >
          Dismiss
        </Button>
      </CardContent>
    </Card>
  {/if}

  {#if a.doctorRun.report}
    <DoctorReportCard report={a.doctorRun.report} />
  {/if}

  <Tabs bind:value={tab}>
    <TabsList>
      <TabsTrigger value="catalog">Catalog</TabsTrigger>
      <TabsTrigger value="history"
        >History ({a.runHistory.length}){#if failedRuns > 0}<span
            class="ml-1.5 rounded-full bg-destructive/15 px-1.5 text-xs text-destructive"
            >{failedRuns} failed</span
          >{/if}</TabsTrigger
      >
      <TabsTrigger value="generate">Generate</TabsTrigger>
      <TabsTrigger value="release">Release</TabsTrigger>
      {#if showGithub}<TabsTrigger value="github">GitHub runs</TabsTrigger>{/if}
    </TabsList>

    <TabsContent value="catalog" class="mt-4">
      <ActionCatalogSidebar
        actions={a.filteredActions}
        workflows={a.filteredWorkflows}
        warnings={a.warnings}
        loading={a.loading}
        running={a.running}
        selected={a.selected}
        bind:sourceFilter={a.sourceFilter}
        onToggleSelect={a.toggleSelect}
        onRun={(id, e) => void a.runTarget(id, e)}
      />
    </TabsContent>

    <TabsContent value="history" class="mt-4">
      <div class="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,24rem)_minmax(0,1fr)]">
        <div class="min-w-0">
          <RunHistoryCard
            runs={a.runHistory}
            loading={a.historyLoading}
            selectedId={a.selectedRunId}
            onRefresh={() => void a.loadHistory()}
            onSelect={(e, opts) => a.openExecution(e.id, opts)}
            onStarted={(id) => {
              a.openExecution(id);
              void a.loadHistory();
            }}
            onChanged={(deletedId) => {
              if (deletedId && deletedId === a.selectedRunId) a.selectedRunId = null;
              void a.loadHistory();
            }}
          />
        </div>
        <div class="min-w-0">
          {#if a.selectedRunId && a.selectedKind && a.selectedKind !== 'action'}
            <PipelineRunDetail
              executionId={a.selectedRunId}
              onClose={() => (a.selectedRunId = null)}
            />
          {:else if a.selectedRunId}
            <ActionRunDetail
              executionId={a.selectedRunId}
              bind:terminal={a.runTerminal}
              onStarted={(id) => {
                a.openExecution(id);
                void a.loadHistory();
              }}
              onSelectExecution={(id) => {
                a.pinnedRun = true;
                a.openExecution(id);
              }}
              onChanged={() => void a.loadHistory()}
              onClose={() => (a.selectedRunId = null)}
            />
          {:else if a.lastResult}
            <LastRunCard result={a.lastResult} onOpenExecution={(id) => a.openExecution(id)} />
          {:else}
            <p class="py-8 text-center text-sm text-muted-foreground">
              Select a run to see its output.
            </p>
          {/if}
        </div>
      </div>
    </TabsContent>

    <TabsContent value="generate" class="mt-4">
      <PipelineGeneratorCard
        bind:requirement={a.requirement}
        run={a.genRun}
        running={a.genRunning}
        blocked={a.running || a.doctorBusy}
        onGenerate={() => void a.generate()}
        onOpenThread={(id) => void a.openThread(id)}
      />
    </TabsContent>

    <TabsContent value="release" class="mt-4">
      <ReleaseCard {project} />
    </TabsContent>

    {#if showGithub}
      <TabsContent value="github" class="mt-4">
        <GitHubProjectActionsPanel {project} {enabled} />
      </TabsContent>
    {/if}
  </Tabs>
</div>
