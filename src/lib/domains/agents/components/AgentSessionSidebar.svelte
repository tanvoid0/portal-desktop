<!-- Project-grouped session list for the Agents workspace (#546). Reuses the
     domain-neutral AISessionSidebar/AISessionCard shell + the grouping logic
     already written for the old sessions table (`groupSessions.ts`), overlaid
     with live status from the `agentRuns` module store. -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import { cn } from '$lib/utils';
  import AISessionSidebar from '$lib/domains/ai/components/shared/AISessionSidebar.svelte';
  import AISessionCard from '$lib/domains/ai/components/shared/AISessionCard.svelte';
  import { agentSessionsQuery } from '../queries/agentsQueries.js';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import {
    groupSessionsByProject,
    sessionSubpath,
    type SessionGroup,
  } from '../utils/groupSessions.js';
  import type { AgentSessionSummary } from '../generated';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import History from '~icons/fa6-solid/clock-rotate-left';

  interface Props {
    focusedId?: string | null;
    onSelect: (id: string) => void;
    onNewRun: () => void;
  }

  let { focusedId = null, onSelect, onNewRun }: Props = $props();

  const sessionsQuery = createQuery(agentSessionsQuery);
  const sessions = $derived(sessionsQuery.data ?? []);
  const projectsQuery = createProjectsQuery();
  const projects = $derived(projectsQuery.data ?? []);
  const groups = $derived(groupSessionsByProject(sessions, projects));

  function matchesSearch(session: AgentSessionSummary, query: string): boolean {
    if (!query.trim()) return true;
    const needle = query.toLowerCase();
    return (
      (session.title ?? session.backend_id).toLowerCase().includes(needle) ||
      session.cwd.toLowerCase().includes(needle)
    );
  }

  let searchValue = $state('');
  const visibleGroups = $derived.by(() => {
    if (!searchValue.trim()) return groups;
    return groups
      .map((g) => ({ ...g, sessions: g.sessions.filter((s) => matchesSearch(s, searchValue)) }))
      .filter((g) => g.sessions.length > 0);
  });

  // Collapsed state persists per group key — nice-to-have, never blocks the list.
  const STORAGE_KEY = 'portal-agents-collapsed-groups';
  let collapsed = $state<Record<string, boolean>>(loadCollapsed());

  function loadCollapsed(): Record<string, boolean> {
    try {
      return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
    } catch {
      return {};
    }
  }

  function setGroupOpen(key: string, open: boolean) {
    collapsed = { ...collapsed, [key]: !open };
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(collapsed));
    } catch {
      /* best-effort persistence only */
    }
  }

  function subtitleFor(session: AgentSessionSummary, group: SessionGroup): string | undefined {
    return sessionSubpath(session, group) ?? undefined;
  }
</script>

<AISessionSidebar
  title="Agents"
  searchPlaceholder="Search sessions…"
  bind:searchValue
  onCreateNew={onNewRun}
  createTitle="New run"
  loading={sessionsQuery.isPending}
  showLoadingSkeleton={sessionsQuery.isPending}
  isEmpty={visibleGroups.length === 0}
>
  {#snippet empty()}
    <div
      class="flex flex-1 flex-col items-center justify-center gap-2 p-6 text-center text-sm text-muted-foreground"
    >
      <History class="h-5 w-5" />
      <p>No runs yet.</p>
      <Button variant="link" size="sm" class="h-auto p-0" onclick={onNewRun}>Start one</Button>
    </div>
  {/snippet}

  {#each visibleGroups as group (group.key)}
    {@const isOpen = !collapsed[group.key]}
    <Collapsible open={isOpen} onOpenChange={(v: boolean) => setGroupOpen(group.key, v)}>
      <CollapsibleTrigger
        class={cn(
          buttonVariants({ variant: 'ghost' }),
          'h-auto min-h-10 w-full min-w-0 justify-start gap-2 px-2'
        )}
      >
        <ChevronDown
          class="h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform {isOpen
            ? 'rotate-180'
            : ''}"
        />
        <FolderOpen class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        <span class="min-w-0 flex-1 truncate text-left text-xs font-medium">{group.label}</span>
        <span class="shrink-0 text-xs text-muted-foreground tabular-nums">
          {group.sessions.length}
        </span>
      </CollapsibleTrigger>
      <CollapsibleContent class="space-y-0.5 py-0.5 pl-1">
        {#each group.sessions as session (session.id)}
          <AISessionCard
            title={session.title || session.backend_id}
            subtitle={subtitleFor(session, group)}
            isActive={focusedId === session.id}
            isRunning={agentRuns.runningIds.has(session.id)}
            updatedAt={session.ended_at ?? session.started_at}
            trailingBadges={agentRuns.pendingCountFor(session.id) > 0
              ? [{ label: 'Needs approval', variant: 'destructive' }]
              : []}
            onClick={() => onSelect(session.id)}
          />
        {/each}
      </CollapsibleContent>
    </Collapsible>
  {/each}
</AISessionSidebar>
