<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Progress } from "$lib/components/ui/progress";
  import { taskUi } from "../state/taskUi.svelte";
  import type { Task } from "../types";
  import Icon from "@iconify/svelte";

  interface Props {
    onStartTracking?: (taskId: string) => void;
    onStopTracking?: () => void;
  }

  let { onStartTracking, onStopTracking }: Props = $props();

  let elapsedTime = $state(0);
  let intervalId: ReturnType<typeof setInterval> | null = null;
  let currentlyTrackingId = $derived(taskUi.timeTrackingSession?.taskId || null);

  // Update elapsed time every second when tracking
  $effect(() => {
    if (taskUi.currentlyTracking && taskUi.timeTrackingSession?.isActive) {
      intervalId = setInterval(() => {
        const now = new Date();
        const startTime = new Date(taskUi.timeTrackingSession!.startTime);
        elapsedTime = Math.floor((now.getTime() - startTime.getTime()) / 1000);
      }, 1000);
    } else {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
      elapsedTime = 0;
    }
  });

  function formatTime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${minutes}:${secs.toString().padStart(2, "0")}`;
  }

  function handleStartTracking() {
    // This would typically open a task selector
    // For now, we'll use a placeholder
    onStartTracking?.("task-id");
  }

  function handleStopTracking() {
    onStopTracking?.();
  }

  function handlePauseTracking() {
    // Pause functionality would be implemented here
    console.log("Pause tracking");
  }
</script>

<Card class="w-full">
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Icon icon="mdi:timer" class="h-5 w-5" />
      Time Tracker
    </CardTitle>
  </CardHeader>
  <CardContent class="p-4 pt-0">
    {#if taskUi.currentlyTracking && taskUi.timeTrackingSession?.isActive}
      <!-- Active Tracking -->
      <div class="space-y-4">
        <div class="text-center">
          <div class="font-mono text-3xl font-bold text-primary">
            {formatTime(elapsedTime)}
          </div>
          <div class="mt-1 text-sm text-muted-foreground">
            Tracking: Task {currentlyTrackingId || "Unknown"}
          </div>
        </div>

        <!-- Progress Bar removed - estimatedTime not available in TimeTrackingSession -->

        <!-- Action Buttons -->
        <div class="flex gap-2">
          <Button
            variant="destructive"
            onclick={handleStopTracking}
            class="flex-1"
          >
            <Icon icon="mdi:stop" class="mr-2 h-4 w-4" />
            Stop
          </Button>
          <Button
            variant="outline"
            onclick={handlePauseTracking}
            class="flex-1"
          >
            <Icon icon="mdi:pause" class="mr-2 h-4 w-4" />
            Pause
          </Button>
        </div>
      </div>
    {:else}
      <!-- No Active Tracking -->
      <div class="space-y-4">
        <div class="text-center">
          <Icon
            icon="mdi:timer-outline"
            class="mx-auto mb-2 h-12 w-12 text-muted-foreground"
          />
          <div class="text-sm text-muted-foreground">
            No active time tracking
          </div>
        </div>

        <Button variant="default" onclick={handleStartTracking} class="w-full">
          <Icon icon="mdi:play" class="mr-2 h-4 w-4" />
          Start Tracking
        </Button>
      </div>
    {/if}

    <!-- Recent Sessions (placeholder) -->
    {#if !taskUi.currentlyTracking}
      <div class="divider-edge-t divider-edge-full mt-4 pt-4">
        <div class="mb-2 text-xs text-muted-foreground">Recent Sessions</div>
        <div class="space-y-1">
          <div class="flex justify-between text-xs">
            <span>Task Name</span>
            <span>2h 30m</span>
          </div>
          <div class="flex justify-between text-xs">
            <span>Another Task</span>
            <span>1h 15m</span>
          </div>
        </div>
      </div>
    {/if}
  </CardContent>
</Card>
