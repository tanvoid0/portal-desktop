<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { taskUi } from "../state/taskUi.svelte";
</script>

<Card class="h-full">
  <CardContent class="flex h-full flex-col justify-center p-3">
    <div class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-foreground">Progress</span>
        <span class="text-xs font-medium text-foreground"
          >{taskUi.taskStats.completionPercentage}%</span
        >
      </div>
      <div class="h-2 w-full rounded-full bg-muted">
        <div
          class="h-2 rounded-full bg-gradient-to-r from-primary to-success-500 transition-all duration-300"
          style="width: {taskUi.taskStats.completionPercentage}%"
        ></div>
      </div>
      <div class="text-center text-xs text-muted-foreground">
        {taskUi.taskStats.completed} of {taskUi.taskStats.total} completed
      </div>
    </div>
  </CardContent>
</Card>
