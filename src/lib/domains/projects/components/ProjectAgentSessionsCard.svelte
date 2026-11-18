<script lang="ts">
  import Bolt from '~icons/fa6-solid/bolt';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { openAgentsSessionById } from '$lib/domains/agents/utils/launch';
  import type { Project } from '$lib/domains/projects/types';
  import { launchInAgents } from '$lib/domains/projects/utils/launch';

  interface Props {
    project: Project;
    sessions: { id: string; title?: string | null; backend_id: string }[];
  }
  let { project, sessions }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Bolt class="h-5 w-5" />
      Agent sessions
    </CardTitle>
  </CardHeader>
  <CardContent class="space-y-1.5">
    {#if sessions.length === 0}
      <p class="text-sm text-muted-foreground">
        No agent sessions yet. Start one to work on this project with an agent.
      </p>
    {:else}
      {#each sessions as session (session.id)}
        <Button
          variant="link"
          size="sm"
          class="h-auto w-full justify-start truncate p-0 text-left font-normal"
          onclick={() => openAgentsSessionById(session.id)}
        >
          {session.title || session.backend_id}
        </Button>
      {/each}
    {/if}
    <Button variant="outline" size="sm" class="mt-2 gap-2" onclick={() => launchInAgents(project)}>
      <Bolt class="h-3.5 w-3.5" />
      New run
    </Button>
  </CardContent>
</Card>
