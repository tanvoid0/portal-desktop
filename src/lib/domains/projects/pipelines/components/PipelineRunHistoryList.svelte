<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { PageEmpty, PageLoading } from "$lib/components/shell";
  import type { PipelineExecutionListItem } from "../types";
  import {
    formatExecutionDuration,
    getExecutionStatusBadgeVariant,
    getExecutionStatusColor,
    getExecutionStatusIcon,
  } from "../utils/executionDisplay";
  import { History, RefreshCw } from "@lucide/svelte";

  interface Props {
    executions: PipelineExecutionListItem[];
    loading?: boolean;
    title?: string;
    description?: string;
    showProject?: boolean;
    showPipeline?: boolean;
    emptyTitle?: string;
    emptyDescription?: string;
    onRefresh?: () => void;
    onSelect?: (execution: PipelineExecutionListItem) => void;
  }

  let {
    executions,
    loading = false,
    title = "Run history",
    description = "Recent pipeline executions",
    showProject = false,
    showPipeline = true,
    emptyTitle = "No pipeline runs yet",
    emptyDescription = "Run a pipeline to see execution history here.",
    onRefresh,
    onSelect,
  }: Props = $props();
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between gap-2">
      <div>
        <CardTitle class="flex items-center gap-2">
          <History class="h-5 w-5" />
          {title}
        </CardTitle>
        <CardDescription>{description}</CardDescription>
      </div>
      {#if onRefresh}
        <Button variant="ghost" size="sm" onclick={onRefresh} disabled={loading}>
          <RefreshCw
            class="h-4 w-4 {loading ? 'animate-spin' : ''}"
          />
        </Button>
      {/if}
    </div>
  </CardHeader>
  <CardContent>
    {#if loading && executions.length === 0}
      <PageLoading message="Loading run history..." />
    {:else if executions.length === 0}
      <PageEmpty
        title={emptyTitle}
        description={emptyDescription}
        icon={History}
      />
    {:else}
      <div class="space-y-2">
        {#each executions as execution (execution.id)}
          {@const StatusIcon = getExecutionStatusIcon(execution.status)}
          <Button
            type="button"
            variant="outline"
            class="h-auto w-full justify-start rounded-lg border p-3 text-left transition-colors hover:bg-muted/50"
            onclick={() => onSelect?.(execution)}
            disabled={!onSelect}
          >
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div class="flex min-w-0 items-center gap-2">
                <StatusIcon
                  class="h-4 w-4 shrink-0 {getExecutionStatusColor(execution.status)}{execution.status === 'running' ? ' animate-spin' : ''}"
                />
                <Badge variant={getExecutionStatusBadgeVariant(execution.status)}>
                  {execution.status}
                </Badge>
                {#if showPipeline && execution.pipelineName}
                  <span class="truncate text-sm font-medium">
                    {execution.pipelineName}
                  </span>
                {/if}
              </div>
              <span class="text-xs text-muted-foreground">
                {execution.startedAt.toLocaleString()}
              </span>
            </div>

            <div
              class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
            >
              {#if showProject && execution.projectName}
                <span>{execution.projectName}</span>
              {/if}
              <span>
                Duration: {formatExecutionDuration(
                  execution.startedAt,
                  execution.finishedAt,
                )}
              </span>
              {#if execution.error}
                <span class="truncate text-destructive">{execution.error}</span>
              {/if}
            </div>
          </Button>
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>
