<script lang="ts">
  import { onDestroy } from "svelte";
  import { automationStore } from "../stores/automationStore";
  import type { WorkflowExecution } from "../types";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Play,
    Pause,
    CheckCircle,
    XCircle,
    Clock,
    Loader2,
  } from "@lucide/svelte";

  export let executionId: string;
  export let onStatusChange: (execution: WorkflowExecution) => void = () => {};

  let execution: WorkflowExecution | null = null;
  let pollingInterval: ReturnType<typeof setInterval> | null = null;

  $: if (executionId) {
    startPolling();
  }

  async function startPolling() {
    if (!executionId) return;
    await checkStatus();
    pollingInterval = setInterval(checkStatus, 2000) as ReturnType<
      typeof setInterval
    >;
  }

  async function checkStatus() {
    if (!executionId) return;

    try {
      const exec = await automationStore.checkWorkflowStatus(executionId);
      execution = exec;
      onStatusChange(exec);

      if (
        exec.status === "success" ||
        exec.status === "error" ||
        exec.status === "canceled"
      ) {
        stopPolling();
      }
    } catch (error) {
      console.error("Failed to check workflow status:", error);
    }
  }

  function stopPolling() {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
  }

  onDestroy(() => {
    stopPolling();
  });

  function getStatusIcon(status: string) {
    switch (status) {
      case "running":
        return Loader2;
      case "success":
        return CheckCircle;
      case "error":
        return XCircle;
      case "waiting":
        return Clock;
      case "canceled":
        return Pause;
      default:
        return Play;
    }
  }

  function getStatusVariant(
    status: string,
  ): "default" | "secondary" | "destructive" | "outline" {
    switch (status) {
      case "success":
        return "default";
      case "error":
        return "destructive";
      default:
        return "secondary";
    }
  }

  function getStatusText(status: string) {
    switch (status) {
      case "running":
        return "Running";
      case "success":
        return "Completed";
      case "error":
        return "Failed";
      case "waiting":
        return "Waiting";
      case "canceled":
        return "Canceled";
      default:
        return "Unknown";
    }
  }
</script>

{#if execution}
  <Card>
    <CardContent class="p-4">
      <div class="flex items-center space-x-3">
        {#if execution.status === "running"}
          <Loader2 class="h-5 w-5 animate-spin text-primary" />
        {:else}
          {@const StatusIcon = getStatusIcon(execution.status)}
          <StatusIcon class="h-5 w-5" />
        {/if}

        <div class="flex-1">
          <div class="flex items-center gap-2">
            <span class="font-medium">{getStatusText(execution.status)}</span>
            <Badge variant={getStatusVariant(execution.status)}
              >{execution.status}</Badge
            >
          </div>
          <div class="text-sm text-muted-foreground">
            Execution ID: {execution.id}
          </div>
        </div>
      </div>

      {#if execution.started_at}
        <div class="mt-2 text-xs text-muted-foreground">
          Started: {new Date(execution.started_at).toLocaleString()}
        </div>
      {/if}

      {#if execution.finished_at}
        <div class="mt-1 text-xs text-muted-foreground">
          Finished: {new Date(execution.finished_at).toLocaleString()}
        </div>
      {/if}
    </CardContent>
  </Card>
{/if}
