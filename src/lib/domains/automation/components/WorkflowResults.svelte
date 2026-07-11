<script lang="ts">
  import type { WorkflowResult } from "../types";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    CheckCircle,
    XCircle,
    Clock,
    FileText,
    Terminal,
  } from "@lucide/svelte";

  export let result: WorkflowResult;

  function formatDuration(duration: number): string {
    if (duration < 60) {
      return `${duration.toFixed(1)}s`;
    }
    const minutes = Math.floor(duration / 60);
    const seconds = duration % 60;
    return `${minutes}m ${seconds.toFixed(1)}s`;
  }
</script>

<Card>
  <CardHeader class="divider-edge-b divider-edge-full pb-4">
    <div class="flex items-center justify-between gap-4">
      <CardTitle class="flex items-center gap-2 text-lg">
        {#if result.success}
          <CheckCircle class="h-5 w-5 text-green-600" />
        {:else}
          <XCircle class="h-5 w-5 text-destructive" />
        {/if}
        Workflow Results
      </CardTitle>
      <Badge variant={result.success ? "default" : "destructive"}>
        {result.success ? "Success" : "Failed"}
      </Badge>
    </div>
    <p class="text-sm text-muted-foreground">
      Execution ID: {result.execution_id}
    </p>
  </CardHeader>

  <CardContent class="space-y-4 pt-4">
    <div class="flex flex-wrap items-center gap-4 text-sm text-muted-foreground">
      <div class="flex items-center gap-1">
        <Clock class="h-4 w-4" />
        <span>Duration: {formatDuration(result.results.duration)}</span>
      </div>
      <div class="flex items-center gap-1">
        <Terminal class="h-4 w-4" />
        <span>Commands: {result.results.commands_executed.length}</span>
      </div>
    </div>

    {#if result.results.commands_executed.length > 0}
      <div>
        <h4 class="mb-2 font-medium">Commands Executed</h4>
        <div class="space-y-1">
          {#each result.results.commands_executed as command (command)}
            <div
              class="flex items-center gap-2 rounded-md bg-muted p-2 font-mono text-sm"
            >
              <Terminal class="h-4 w-4 text-muted-foreground" />
              <span>{command}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if result.results.output}
      <div>
        <h4 class="mb-2 font-medium">Output</h4>
        <pre
          class="overflow-x-auto whitespace-pre-wrap rounded-md bg-muted p-3 font-mono text-sm">{result.results.output}</pre>
      </div>
    {/if}

    {#if result.results.files_created.length > 0}
      <div>
        <h4 class="mb-2 font-medium">Files Created</h4>
        <div class="space-y-1">
          {#each result.results.files_created as file (file)}
            <div
              class="flex items-center gap-2 rounded-md border border-green-500/30 bg-green-500/10 p-2 text-sm"
            >
              <FileText class="h-4 w-4 text-green-600" />
              <span>{file}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if result.errors.length > 0}
      <div>
        <h4 class="mb-2 font-medium text-destructive">Errors</h4>
        <div class="space-y-1">
          {#each result.errors as error (error)}
            <div
              class="rounded-md border border-destructive/30 bg-destructive/10 p-2 text-sm text-destructive"
            >
              {error}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if result.suggestions.length > 0}
      <div>
        <h4 class="mb-2 font-medium">Suggestions</h4>
        <div class="space-y-1">
          {#each result.suggestions as suggestion (suggestion)}
            <div class="rounded-md border bg-muted/50 p-2 text-sm">
              {suggestion}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </CardContent>
</Card>
