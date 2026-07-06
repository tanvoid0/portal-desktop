<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import Icon from "@iconify/svelte";
  import { taskUi } from "../state/taskUi.svelte";
</script>

<div class="grid grid-cols-2 gap-2 md:grid-cols-4">
  <Card>
    <CardContent class="p-2">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-medium text-muted-foreground">Total</p>
          <p class="text-sm font-bold text-foreground">{taskUi.taskStats.total}</p>
        </div>
        <Icon icon="mdi:clipboard-text" class="h-4 w-4 text-primary" />
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardContent class="p-2">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-medium text-muted-foreground">To Do</p>
          <p class="text-sm font-bold text-foreground">{taskUi.taskStats.pending}</p>
        </div>
        <Icon icon="mdi:clock-outline" class="h-4 w-4 text-muted-foreground" />
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardContent class="p-2">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-medium text-muted-foreground">In Progress</p>
          <p class="text-sm font-bold text-foreground">
            {taskUi.taskStats.inProgress}
          </p>
        </div>
        <Icon icon="mdi:progress-clock" class="h-4 w-4 text-primary" />
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardContent class="p-2">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-medium text-muted-foreground">Done</p>
          <p class="text-sm font-bold text-foreground">
            {taskUi.taskStats.completed}
          </p>
        </div>
        <Icon icon="mdi:check-circle" class="h-4 w-4 text-success-500" />
      </div>
    </CardContent>
  </Card>
</div>
