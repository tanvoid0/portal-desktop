<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
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
      <Button
        type="button"
        size="sm"
        class="gap-2"
        onclick={() => void a.runSelected()}
        disabled={a.running || a.selected.size === 0}
      >
        <Play class="h-4 w-4" />
        {a.running ? 'Running…' : `Run selected (${a.selected.size})`}
      </Button>
    </div>
  </div>

  <div class="flex flex-col gap-6 xl:flex-row xl:items-start">
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

    <!-- Run details + history -->
    <div class="min-w-0 flex-1 space-y-4">
      <PipelineGeneratorCard
        bind:requirement={a.requirement}
        run={a.genRun}
        running={a.genRunning}
        blocked={a.running || a.doctorBusy}
        onGenerate={() => void a.generate()}
        onOpenThread={(id) => void a.openThread(id)}
      />

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

      <div class="grid gap-4 min-[2200px]:grid-cols-2 min-[2200px]:items-start">
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
        {/if}

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
    </div>
  </div>

  <ReleaseCard {project} />

  {#if a.sourceFilter === 'all' || a.sourceFilter === 'github'}
    <div class="space-y-3 border-t pt-6">
      <h3 class="text-sm font-medium text-muted-foreground">GitHub Actions runs</h3>
      <GitHubProjectActionsPanel {project} {enabled} />
    </div>
  {/if}
</div>
