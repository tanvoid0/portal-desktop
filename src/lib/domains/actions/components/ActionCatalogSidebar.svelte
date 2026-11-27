<!-- Catalog tab of Actions: source filter, catalog warnings, workflow and action cards with Run. -->
<script lang="ts">
  import * as Collapsible from '$lib/components/ui/collapsible';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { PageEmpty, PageLoading } from '$lib/components/shell';
  import Play from '~icons/fa6-solid/play';
  import Workflow from '~icons/fa6-solid/diagram-project';
  import Terminal from '~icons/fa6-solid/terminal';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import Layers from '~icons/fa6-solid/layer-group';
  import type { UnifiedAction, UnifiedWorkflow, ActionSource } from '$lib/domains/actions';
  import { dedupePlan } from '../stepGraph';
  import WorkflowTree from './WorkflowTree.svelte';

  type SourceFilter = 'all' | ActionSource;

  interface Props {
    actions: UnifiedAction[];
    workflows: UnifiedWorkflow[];
    warnings: string[];
    loading: boolean;
    running: boolean;
    selected: Set<string>;
    sourceFilter: SourceFilter;
    onToggleSelect: (id: string) => void;
    onRun: (target: string, event: Event) => void;
  }

  let {
    actions,
    workflows,
    warnings,
    loading,
    running,
    selected,
    sourceFilter = $bindable(),
    onToggleSelect,
    onRun,
  }: Props = $props();

  const FILTERS: { id: SourceFilter; label: string }[] = [
    { id: 'all', label: 'All' },
    { id: 'local', label: 'Local' },
    { id: 'file', label: 'File' },
    { id: 'github', label: 'GitHub' },
    { id: 'n8n', label: 'n8n' },
  ];

  function sourceIcon(source: ActionSource) {
    if (source === 'github') return GitBranch;
    if (source === 'n8n') return Layers;
    if (source === 'file') return Workflow;
    return Terminal;
  }
</script>

<aside class="w-full min-w-0 space-y-4">
  <div class="flex flex-wrap gap-2">
    {#each FILTERS as filter (filter.id)}
      <Button
        size="sm"
        variant={sourceFilter === filter.id ? 'default' : 'outline'}
        onclick={() => (sourceFilter = filter.id)}
      >
        {filter.label}
      </Button>
    {/each}
  </div>

  {#if warnings.length > 0}
    <Collapsible.Root
      class="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-1.5 text-sm"
    >
      <Collapsible.Trigger
        class="group/warn flex w-full cursor-pointer items-center gap-1.5 text-left text-amber-700 select-none dark:text-amber-400"
      >
        <ChevronRight class="h-3 w-3 transition-transform group-data-[state=open]/warn:rotate-90" />
        {warnings.length}
        {warnings.length === 1 ? 'source unavailable' : 'sources unavailable'}
      </Collapsible.Trigger>
      <Collapsible.Content class="space-y-1 py-1.5">
        {#each warnings as warning (warning)}
          <p>{warning}</p>
        {/each}
      </Collapsible.Content>
    </Collapsible.Root>
  {/if}

  {#if loading}
    <PageLoading message="Loading actions catalog…" />
  {:else if actions.length === 0 && workflows.length === 0}
    <PageEmpty
      title="No actions available"
      description="Add framework/package manager metadata, create a .portal/pipeline.yml, or use the Generate tab to have an agent write one."
      icon={Workflow}
    />
  {:else}
    {#if workflows.length > 0}
      <div class="space-y-2">
        <h3 class="text-sm font-medium text-muted-foreground">Workflows</h3>
        <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
          {#each workflows as workflow (workflow.id)}
            {@const Icon = sourceIcon(workflow.source)}
            <Card>
              <CardHeader class="pb-2">
                <div class="flex items-start justify-between gap-2">
                  <CardTitle class="text-sm">{workflow.name}</CardTitle>
                  <Badge variant="secondary">{workflow.source}</Badge>
                </div>
                {#if workflow.description}
                  <CardDescription class="text-xs">
                    {workflow.description}
                  </CardDescription>
                {/if}
              </CardHeader>
              <CardContent class="space-y-2">
                {#if workflow.steps.length > 0}
                  <WorkflowTree
                    plan={dedupePlan(
                      workflow.steps.map((s) => ({
                        id: s.action,
                        name: s.action,
                        needs: s.needs ?? [],
                      }))
                    )}
                  />
                {:else}
                  <p class="flex min-w-0 items-center gap-1 truncate text-xs text-muted-foreground">
                    <Icon class="h-3.5 w-3.5 shrink-0" />
                    {workflow.runner}
                  </p>
                {/if}
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  class="gap-1"
                  disabled={running}
                  onclick={(e) => onRun(workflow.id, e)}
                >
                  <Play class="h-3.5 w-3.5" />
                  Run
                </Button>
              </CardContent>
            </Card>
          {/each}
        </div>
      </div>
    {/if}

    <div class="space-y-2">
      <h3 class="text-sm font-medium text-muted-foreground">Actions</h3>
      <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4">
        {#each actions as action (action.id)}
          {@const Icon = sourceIcon(action.source)}
          <Card>
            <CardHeader class="pb-2">
              <div class="flex items-start gap-2">
                <Checkbox
                  checked={selected.has(action.id)}
                  onCheckedChange={() => onToggleSelect(action.id)}
                  class="mt-1"
                />
                <div class="min-w-0 flex-1">
                  <div class="flex items-start justify-between gap-2">
                    <CardTitle class="text-sm">{action.name}</CardTitle>
                    <Badge variant="outline">{action.source}</Badge>
                  </div>
                  {#if action.description}
                    <CardDescription class="text-xs">
                      {action.description}
                    </CardDescription>
                  {/if}
                </div>
              </div>
            </CardHeader>
            <CardContent class="space-y-2">
              {#if action.command}
                <code class="block truncate rounded bg-muted px-2 py-1 text-xs">
                  {action.command}
                </code>
              {:else}
                <p class="flex items-center gap-1 text-xs text-muted-foreground">
                  <Icon class="h-3.5 w-3.5" />
                  {action.runner} runner
                </p>
              {/if}
              <Button
                type="button"
                size="sm"
                class="w-full gap-1"
                disabled={running}
                onclick={(e) => onRun(action.id, e)}
              >
                <Play class="h-3.5 w-3.5" />
                Run
              </Button>
            </CardContent>
          </Card>
        {/each}
      </div>
    </div>
  {/if}
</aside>
