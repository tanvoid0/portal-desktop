<script lang="ts">
  import FaClipboard from '~icons/fa6-solid/clipboard';
  import FaClock from '~icons/fa6-solid/clock';
  import FaHourglassHalf from '~icons/fa6-solid/hourglass-half';
  import FaCircleCheck from '~icons/fa6-solid/circle-check';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { taskUi } from '../state/taskUi.svelte';
</script>

<div class="grid grid-cols-2 gap-2 md:grid-cols-4">
  <Card>
    <CardContent class="p-2">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-medium text-muted-foreground">Total</p>
          <p class="text-sm font-bold text-foreground">{taskUi.taskStats.total}</p>
        </div>
        <FaClipboard class="h-4 w-4 text-primary" />
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
        <FaClock class="h-4 w-4 text-muted-foreground" />
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
        <FaHourglassHalf class="h-4 w-4 text-primary" />
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
        <FaCircleCheck class="h-4 w-4 text-success-500" />
      </div>
    </CardContent>
  </Card>
</div>
