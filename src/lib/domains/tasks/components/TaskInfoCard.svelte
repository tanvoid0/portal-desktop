<script lang="ts">
  import FaCalendarPlus from '~icons/fa6-solid/calendar-plus';
  import FaCalendarDays from '~icons/fa6-solid/calendar-days';
  import FaCircleCheck from '~icons/fa6-solid/circle-check';
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import type { Task } from '../types';

  interface Props {
    task: Task;
    /** Shows the parent task; the button is hidden when it is not supplied. */
    onOpenTask?: (taskId: string) => void;
  }

  let { task, onOpenTask }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle class="text-lg">Task Information</CardTitle>
  </CardHeader>
  <CardContent class="space-y-4">
    <div>
      <Label class="text-sm font-medium text-muted-foreground">Created</Label>
      <div class="mt-1 flex items-center gap-2">
        <FaCalendarPlus class="h-4 w-4 text-muted-foreground" />
        <span class="text-sm">{new Date(task.createdAt).toLocaleDateString()}</span>
      </div>
    </div>

    <div>
      <Label class="text-sm font-medium text-muted-foreground">Last Updated</Label>
      <div class="mt-1 flex items-center gap-2">
        <FaCalendarDays class="h-4 w-4 text-muted-foreground" />
        <span class="text-sm">{new Date(task.updatedAt).toLocaleDateString()}</span>
      </div>
    </div>

    {#if task.completedAt}
      <div>
        <Label class="text-sm font-medium text-muted-foreground">Completed</Label>
        <div class="mt-1 flex items-center gap-2">
          <FaCircleCheck class="h-4 w-4 text-green-500" />
          <span class="text-sm">{new Date(task.completedAt).toLocaleDateString()}</span>
        </div>
      </div>
    {/if}

    {#if task.parentId && onOpenTask}
      <div>
        <Label class="text-sm font-medium text-muted-foreground">Parent Task</Label>
        <div class="mt-1">
          <Button
            variant="outline"
            size="sm"
            onclick={() => onOpenTask?.(task.parentId!)}
            class="w-full justify-start"
          >
            <FaTurnDown class="mr-2 h-4 w-4" />
            View Parent Task
          </Button>
        </div>
      </div>
    {/if}
  </CardContent>
</Card>
