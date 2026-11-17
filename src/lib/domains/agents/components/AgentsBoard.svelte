<!-- Agents board (#191): the shared tasks KanbanBoard fed by sessions, read-only. -->
<script lang="ts">
  import KanbanBoard from '$lib/domains/tasks/components/KanbanBoard.svelte';
  import { createTaskUiState } from '$lib/domains/tasks/state/taskUi.svelte';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { AGENT_COLUMNS, sessionsToTasks } from '../utils/sessionToTask.js';
  import type { AgentSessionSummary } from '../generated';

  interface Props {
    sessions: AgentSessionSummary[];
    backendName: (id: string) => string;
    onOpen: (s: AgentSessionSummary) => void;
  }
  let { sessions, backendName, onOpen }: Props = $props();

  const ui = createTaskUiState({ columns: AGENT_COLUMNS });
  ui.archiveEnabled = false; // sessions are already capped by the overview, not by age

  $effect(() => {
    void agentRuns.ensureInit(); // live events -> waiting/subagent state for running sessions
  });
  $effect(() => {
    ui.setTasks(
      sessionsToTasks(sessions, {
        isWaiting: (id) => agentRuns.pendingCountFor(id) > 0,
        events: (id) => agentRuns.eventsFor(id),
        backendName,
      })
    );
  });

  // A subtask card opens its parent session.
  const open = (t: { id: string; parentId?: string }) => {
    const s = sessions.find((x) => x.id === (t.parentId ?? t.id));
    if (s) onOpen(s);
  };
</script>

<section class="space-y-2">
  <h2 class="text-sm font-medium text-muted-foreground">Board</h2>
  <div class="h-96">
    <KanbanBoard
      {ui}
      readonly
      showSubtaskActions={false}
      handleTaskSelect={open}
      handleTaskStatusToggle={() => {}}
      handleTaskSelection={() => {}}
      handleCreateSubtask={() => {}}
    />
  </div>
</section>
