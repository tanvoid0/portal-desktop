<script lang="ts">
  import FaCalendarPlus from '~icons/fa6-solid/calendar-plus';
  import FaCalendarDays from '~icons/fa6-solid/calendar-days';
  import FaCircleCheck from '~icons/fa6-solid/circle-check';
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFolder from '~icons/fa6-solid/folder';
  import FaBolt from '~icons/fa6-solid/bolt';
  import { goto } from '$app/navigation';
  import { createQuery } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { ResourceType } from '$lib/domains/shared/types/resourceType';
  import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { openAgentsSessionById } from '$lib/domains/agents/utils/launch';
  import type { Task } from '../types';

  interface Props {
    task: Task;
    /** Shows the parent task; the button is hidden when it is not supplied. */
    onOpenTask?: (taskId: string) => void;
  }

  let { task, onOpenTask }: Props = $props();

  const projectsQuery = createProjectsQuery();
  const linkedProject = $derived(
    task.resourceType === ResourceType.PROJECT && task.resourceId
      ? (projectsQuery.data ?? []).find((p) => String(p.id) === task.resourceId)
      : undefined
  );

  // Small list, filtered client-side — no dedicated backend query for "sessions for this task".
  const agentSessionsResult = createQuery(agentSessionsQuery);
  const taskAgentSessions = $derived(
    (agentSessionsResult.data ?? []).filter((s) => s.task_id === task.id)
  );
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

    {#if linkedProject}
      <div>
        <Label class="text-sm font-medium text-muted-foreground">Project</Label>
        <div class="mt-1">
          <Button
            variant="outline"
            size="sm"
            onclick={() => goto(`/projects/${linkedProject.id}`)}
            class="w-full justify-start"
          >
            <FaFolder class="mr-2 h-4 w-4" />
            {linkedProject.name}
          </Button>
        </div>
      </div>
    {/if}

    {#if taskAgentSessions.length > 0}
      <div>
        <Label class="text-sm font-medium text-muted-foreground">Agent sessions</Label>
        <div class="mt-1 space-y-1">
          {#each taskAgentSessions as session (session.id)}
            <Button
              variant="outline"
              size="sm"
              onclick={() => openAgentsSessionById(session.id)}
              class="w-full justify-start"
            >
              <FaBolt class="mr-2 h-4 w-4" />
              {session.title || session.backend_id}
            </Button>
          {/each}
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
