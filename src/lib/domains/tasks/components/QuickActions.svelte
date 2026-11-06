<script lang="ts">
  import FaStopwatch from '~icons/fa6-solid/stopwatch';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaStop from '~icons/fa6-solid/stop';
  import FaPlay from '~icons/fa6-solid/play';
  import FaCircleCheck from '~icons/fa6-solid/circle-check';
  import FaTableCellsLarge from '~icons/fa6-solid/table-cells-large';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { taskUi } from '../state/taskUi.svelte';

  interface Props {
    onQuickAdd?: () => void;
    onStartTracking?: (taskId: string) => void;
    onStopTracking?: () => void;
  }

  let { onQuickAdd, onStartTracking, onStopTracking }: Props = $props();

  // Quick action handlers
  function handleQuickAdd() {
    onQuickAdd?.();
  }

  function handleStartTracking() {
    if (taskUi.currentlyTracking) {
      onStartTracking?.(taskUi.currentlyTracking.taskId);
    }
  }

  function handleStopTracking() {
    onStopTracking?.();
  }

  function handleQuickComplete() {
    // Find a quick task to complete
    // This would be implemented based on your business logic
    console.log('Quick complete task');
  }

  function handleQuickTemplate() {
    // Apply a common template
    console.log('Apply quick template');
  }
</script>

<Card class="w-full">
  <CardContent class="p-4">
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold">Quick Actions</h3>
        {#if taskUi.currentlyTracking}
          <Badge
            variant="secondary"
            class="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300"
          >
            <FaStopwatch class="mr-1 h-3 w-3" />
            Tracking
          </Badge>
        {/if}
      </div>

      <Separator />

      <!-- Quick Action Buttons -->
      <div class="grid grid-cols-2 gap-2">
        <Button
          variant="default"
          onclick={handleQuickAdd}
          class="flex h-12 flex-col items-center gap-1"
        >
          <FaPlus class="h-5 w-5" />
          <span class="text-xs">Quick Add</span>
        </Button>

        {#if taskUi.currentlyTracking}
          <Button
            variant="destructive"
            onclick={handleStopTracking}
            class="flex h-12 flex-col items-center gap-1"
          >
            <FaStop class="h-5 w-5" />
            <span class="text-xs">Stop Timer</span>
          </Button>
        {:else}
          <Button
            variant="secondary"
            onclick={handleStartTracking}
            class="flex h-12 flex-col items-center gap-1"
          >
            <FaPlay class="h-5 w-5" />
            <span class="text-xs">Start Timer</span>
          </Button>
        {/if}

        <Button
          variant="outline"
          onclick={handleQuickComplete}
          class="flex h-12 flex-col items-center gap-1"
        >
          <FaCircleCheck class="h-5 w-5" />
          <span class="text-xs">Quick Complete</span>
        </Button>

        <Button
          variant="outline"
          onclick={handleQuickTemplate}
          class="flex h-12 flex-col items-center gap-1"
        >
          <FaTableCellsLarge class="h-5 w-5" />
          <span class="text-xs">Template</span>
        </Button>
      </div>

      <!-- Productivity Metrics -->
      {#if taskUi.productivityMetrics}
        <div class="space-y-2">
          <h4 class="text-sm font-medium text-muted-foreground">Today's Progress</h4>
          <div class="grid grid-cols-2 gap-2 text-xs">
            <div class="flex justify-between">
              <span>Completion Rate:</span>
              <span class="font-medium"
                >{Math.round(taskUi.productivityMetrics.completionRate * 100)}%</span
              >
            </div>
            <div class="flex justify-between">
              <span>Time Accuracy:</span>
              <span class="font-medium"
                >{Math.round(taskUi.productivityMetrics.timeAccuracy * 100)}%</span
              >
            </div>
            <div class="flex justify-between">
              <span>Velocity:</span>
              <span class="font-medium">{taskUi.productivityMetrics.velocity} tasks</span>
            </div>
            <div class="flex justify-between">
              <span>Overdue:</span>
              <span class="font-medium text-destructive"
                >{taskUi.productivityMetrics.overdueCount}</span
              >
            </div>
          </div>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
