<script lang="ts">
  import FaRotateLeft from '~icons/fa6-solid/rotate-left';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { taskActions } from '../stores/taskStore';
  import { toastActions } from '$lib/utils/toast';
  import { taskStatusLabel, type Task } from '../types';
  import { getStatusBadgeColor } from '../utils/taskDisplay';

  interface Props {
    tasks: Task[];
    onSelect: (task: Task) => void;
    projectLabel?: (task: Task) => string | undefined;
    /** Navigates to the task's linked project when its badge is clicked. */
    onProjectClick?: (task: Task) => void;
  }

  let { tasks, onSelect, projectLabel, onProjectClick }: Props = $props();

  const monthFmt = new Intl.DateTimeFormat('en', { month: 'long', year: 'numeric' });
  const dayFmt = new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric' });

  // `tasks` arrives newest-first, so insertion order keeps months newest-first too.
  const groups = $derived.by(() => {
    const map = new Map<string, Task[]>();
    for (const t of tasks) {
      const key = monthFmt.format(t.completedAt ?? t.updatedAt);
      map.set(key, [...(map.get(key) ?? []), t]);
    }
    return [...map.entries()];
  });

  async function restore(task: Task) {
    try {
      await taskActions.updateTask(task.id, { status: 'pending' });
      toastActions.success('Task restored', `"${task.title}" is back in Backlog`);
    } catch (err) {
      toastActions.error(
        'Failed to restore task',
        err instanceof Error ? err.message : 'An unexpected error occurred'
      );
    }
  }
</script>

<div class="space-y-6">
  {#each groups as [month, items] (month)}
    <section class="space-y-2">
      <h3 class="text-sm font-semibold text-muted-foreground">
        {month} <span class="font-normal">({items.length})</span>
      </h3>
      <Card class="gap-0 py-0">
        <CardContent class="divide-y divide-border p-0">
          {#each items as task (task.id)}
            {@const project = projectLabel?.(task)}
            <div class="flex items-center gap-3 p-3">
              <Button
                variant="ghost"
                class="h-auto min-w-0 flex-1 justify-start p-0 text-left font-medium whitespace-normal hover:bg-transparent"
                onclick={() => onSelect(task)}
              >
                <span class="line-clamp-2 break-words">{task.title}</span>
              </Button>
              {#if project}
                <Badge
                  variant="outline"
                  class={onProjectClick
                    ? 'max-w-[10rem] cursor-pointer truncate hover:bg-accent'
                    : 'max-w-[10rem] truncate'}
                  onclick={onProjectClick
                    ? (e: MouseEvent) => {
                        e.stopPropagation();
                        onProjectClick?.(task);
                      }
                    : undefined}
                >
                  {project}
                </Badge>
              {/if}
              <Badge class={getStatusBadgeColor(task.status)}>{taskStatusLabel(task.status)}</Badge>
              <span class="w-14 shrink-0 text-right text-xs text-muted-foreground">
                {dayFmt.format(task.completedAt ?? task.updatedAt)}
              </span>
              <Button variant="outline" size="sm" class="shrink-0" onclick={() => restore(task)}>
                <FaRotateLeft class="mr-1 h-3 w-3" />
                Restore
              </Button>
            </div>
          {/each}
        </CardContent>
      </Card>
    </section>
  {/each}
</div>
