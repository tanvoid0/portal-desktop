<!--
	Execution Monitor - Vercel-style step timeline with per-step logs
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    CheckCircle2,
    XCircle,
    Loader2,
    Circle,
    Ban,
    Clock,
    Copy,
    Check,
  } from "@lucide/svelte";
  import { toast } from "$lib/utils/toast";
  import type { PipelineExecution, StepExecution } from "../types";
  import { executionService } from "../services/executionService";
  import StepLogViewer from "./StepLogViewer.svelte";

  interface Props {
    executionId: string;
    onClose?: () => void;
  }

  let { executionId, onClose }: Props = $props();

  let execution = $state<PipelineExecution | null>(null);
  let selectedStepId = $state<string | null>(null);
  let liveLogBuffers = $state<Record<string, string[]>>({});
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let unlistenExecution: (() => void) | null = null;
  let unlistenStepLog: (() => void) | null = null;
  let cancelling = $state(false);
  let errorCopied = $state(false);

  const hasRunningStep = $derived(
    execution?.stepExecutions.some((s) => s.status === "running") ?? false,
  );

  const cancelLabel = $derived(hasRunningStep ? "Stop" : "Cancel");

  const selectedStep = $derived(
    execution?.stepExecutions.find((s) => s.stepId === selectedStepId) ?? null,
  );

  const selectedLiveLines = $derived(
    selectedStepId ? (liveLogBuffers[selectedStepId] ?? []) : [],
  );

  function normalizeExecution(raw: PipelineExecution): PipelineExecution {
    return {
      ...raw,
      startedAt: new Date(raw.startedAt),
      finishedAt: raw.finishedAt ? new Date(raw.finishedAt) : undefined,
      stepExecutions: (raw.stepExecutions ?? []).map((s) => ({
        ...s,
        startedAt: new Date(s.startedAt),
        finishedAt: s.finishedAt ? new Date(s.finishedAt) : undefined,
        logs: s.logs ?? [],
      })),
    };
  }

  function syncLiveBuffersFromExecution(exec: PipelineExecution) {
    const next: Record<string, string[]> = { ...liveLogBuffers };
    for (const step of exec.stepExecutions) {
      if (step.logs?.length) {
        next[step.stepId] = [...step.logs];
      } else if (!next[step.stepId]) {
        next[step.stepId] = [];
      }
    }
    liveLogBuffers = next;
  }

  function pickActiveStep(exec: PipelineExecution): string | null {
    const running = exec.stepExecutions.find((s) => s.status === "running");
    if (running) return running.stepId;
    const failed = exec.stepExecutions.find((s) => s.status === "failed");
    if (failed) return failed.stepId;
    if (exec.stepExecutions.length > 0) {
      return exec.stepExecutions[exec.stepExecutions.length - 1].stepId;
    }
    return null;
  }

  async function refreshExecution() {
    const updated = await executionService.getExecution(executionId);
    if (!updated) return;
    execution = normalizeExecution(updated);
    syncLiveBuffersFromExecution(execution);
    if (!selectedStepId) {
      selectedStepId = pickActiveStep(execution);
    }
  }

  onMount(async () => {
    await refreshExecution();

    unlistenExecution = await listen<PipelineExecution>(
      "pipeline-execution-update",
      (event) => {
        if (event.payload.id !== executionId) return;
        execution = normalizeExecution(event.payload);
        syncLiveBuffersFromExecution(execution);
        if (!selectedStepId || execution.stepExecutions.some((s) => s.status === "running")) {
          selectedStepId = pickActiveStep(execution) ?? selectedStepId;
        }
      },
    );

    unlistenStepLog = await listen<{
      executionId: string;
      stepId: string;
      line: string;
      stream: string;
    }>("pipeline-step-log", (event) => {
      if (event.payload.executionId !== executionId) return;
      const { stepId, line, stream } = event.payload;
      const formatted = stream === "stderr" ? `[stderr] ${line}` : line;
      const existing = liveLogBuffers[stepId] ?? [];
      liveLogBuffers = {
        ...liveLogBuffers,
        [stepId]: [...existing, formatted],
      };
      if (!selectedStepId || execution?.stepExecutions.some((s) => s.stepId === stepId && s.status === "running")) {
        selectedStepId = stepId;
      }
    });

    pollInterval = setInterval(async () => {
      if (execution && ["pending", "running"].includes(execution.status)) {
        await refreshExecution();
      } else if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
    }, 1000);
  });

  onDestroy(() => {
    unlistenExecution?.();
    unlistenStepLog?.();
    if (pollInterval) clearInterval(pollInterval);
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case "success":
        return "text-green-500";
      case "failed":
        return "text-red-500";
      case "running":
        return "text-blue-500";
      case "cancelled":
        return "text-yellow-500";
      default:
        return "text-muted-foreground";
    }
  }

  function formatDuration(step: StepExecution): string {
    if (step.duration) {
      return step.duration < 1000
        ? `${step.duration}ms`
        : `${Math.round(step.duration / 1000)}s`;
    }
    return "";
  }

  async function copyError() {
    if (!execution?.error) return;

    try {
      await navigator.clipboard.writeText(execution.error);
      errorCopied = true;
      toast.success("Error copied to clipboard");
      setTimeout(() => {
        errorCopied = false;
      }, 2000);
    } catch {
      toast.error("Failed to copy error");
    }
  }

  async function handleCancel() {
    if (
      !execution ||
      cancelling ||
      !["pending", "running"].includes(execution.status)
    ) {
      return;
    }

    try {
      cancelling = true;
      await executionService.cancelExecution(executionId);
      await refreshExecution();
    } finally {
      cancelling = false;
    }
  }
</script>

<div class="flex h-full flex-col gap-4">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-lg font-semibold">Pipeline Run</h2>
      {#if execution}
        <p class="text-sm text-muted-foreground">
          Started {new Date(execution.startedAt).toLocaleString()}
          {#if execution.finishedAt}
            · Finished {new Date(execution.finishedAt).toLocaleString()}
          {/if}
        </p>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if execution}
        <Badge variant="outline" class={getStatusColor(execution.status)}>
          {execution.status}
        </Badge>
      {/if}
      {#if execution && ["pending", "running"].includes(execution.status)}
        <Button
          variant="destructive"
          size="sm"
          onclick={handleCancel}
          disabled={cancelling}
        >
          {#if cancelling}
            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            Stopping...
          {:else}
            {cancelLabel}
          {/if}
        </Button>
      {/if}
      {#if onClose}
        <Button variant="outline" size="sm" onclick={onClose}>Close</Button>
      {/if}
    </div>
  </div>

  {#if !execution}
    <Card>
      <CardContent class="py-12 text-center text-muted-foreground">
        Loading execution...
      </CardContent>
    </Card>
  {:else}
    <div class="grid min-h-[480px] flex-1 gap-4 lg:grid-cols-[280px_1fr]">
      <!-- Step timeline -->
      <Card class="h-fit">
        <CardHeader class="pb-2">
          <CardTitle class="text-sm font-medium">Steps</CardTitle>
        </CardHeader>
        <CardContent class="space-y-1 p-2">
          {#each execution.stepExecutions as step (step.stepId)}
            <Button
              type="button"
              variant="ghost"
              class="flex h-auto w-full items-start gap-3 rounded-md px-3 py-2 text-left {selectedStepId ===
              step.stepId
                ? 'bg-muted'
                : ''}"
              onclick={() => (selectedStepId = step.stepId)}
            >
              <div class="mt-0.5 shrink-0">
                {#if step.status === "success"}
                  <CheckCircle2 class="h-4 w-4 text-green-500" />
                {:else if step.status === "failed"}
                  <XCircle class="h-4 w-4 text-red-500" />
                {:else if step.status === "running"}
                  <Loader2 class="h-4 w-4 animate-spin text-blue-500" />
                {:else if step.status === "cancelled"}
                  <Ban class="h-4 w-4 text-yellow-500" />
                {:else}
                  <Circle class="h-4 w-4 text-muted-foreground" />
                {/if}
              </div>
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium">{step.stepName}</p>
                <div
                  class="flex items-center gap-2 text-xs text-muted-foreground"
                >
                  <span class={getStatusColor(step.status)}>{step.status}</span
                  >
                  {#if formatDuration(step)}
                    <span class="flex items-center gap-0.5">
                      <Clock class="h-3 w-3" />
                      {formatDuration(step)}
                    </span>
                  {/if}
                </div>
              </div>
            </Button>
          {/each}
        </CardContent>
      </Card>

      <!-- Log panel -->
      <StepLogViewer
        step={selectedStep}
        liveLines={selectedLiveLines}
        autoScroll={execution.status === "running"}
      />
    </div>

    {#if execution.error}
      <div
        class="rounded border border-red-200 bg-red-50 p-3 dark:border-red-800 dark:bg-red-900/20"
      >
        <div class="flex items-start justify-between gap-2">
          <p class="text-sm font-medium text-red-800 dark:text-red-200">Error</p>
          <Button
            variant="ghost"
            size="sm"
            class="h-7 shrink-0 text-red-700 hover:text-red-900 dark:text-red-300 dark:hover:text-red-100"
            onclick={copyError}
            title="Copy error"
          >
            {#if errorCopied}
              <Check class="mr-1 h-3.5 w-3.5" />
              Copied
            {:else}
              <Copy class="mr-1 h-3.5 w-3.5" />
              Copy
            {/if}
          </Button>
        </div>
        <p class="text-sm text-red-700 dark:text-red-300">{execution.error}</p>
      </div>
    {/if}
  {/if}
</div>
