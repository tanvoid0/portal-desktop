<script lang="ts">
  import { goto } from '$app/navigation';
  import { createQuery } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { PageHeader, PageEmpty } from '$lib/components/shell';
  import { confirmAction } from '$lib/utils/confirm';
  import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import NotificationRow from '$lib/domains/notifications/components/NotificationRow.svelte';
  import {
    notificationInbox,
    type InboxItem,
  } from '$lib/domains/notifications/state/notificationInbox.svelte';
  import Bell from '~icons/fa6-solid/bell';
  import CheckDouble from '~icons/fa6-solid/check-double';
  import Trash from '~icons/fa6-solid/trash';

  const sessionsQuery = createQuery(agentSessionsQuery);
  const projectsQuery = createProjectsQuery();

  let unreadOnly = $state(false);
  const unread = $derived(notificationInbox.unread);
  const visible = $derived(
    unreadOnly ? notificationInbox.items.filter((i) => !i.read) : notificationInbox.items
  );

  function open(item: InboxItem) {
    notificationInbox.markRead(item.id);
    if (item.route) void goto(item.route);
  }

  async function clearAll() {
    if (await confirmAction('Remove every notification from this list?', 'Clear notifications')) {
      notificationInbox.clear();
    }
  }
</script>

<div class="flex flex-col gap-4">
  <PageHeader title="Notifications" description="Agent, pipeline and device events, newest first.">
    {#snippet actions()}
      <Button
        variant="outline"
        class="gap-2"
        disabled={!unread}
        onclick={() => notificationInbox.markAllRead()}
      >
        <CheckDouble class="h-3.5 w-3.5" />
        Mark all read
      </Button>
      <Button
        variant="outline"
        class="gap-2"
        disabled={!notificationInbox.items.length}
        onclick={clearAll}
      >
        <Trash class="h-3.5 w-3.5" />
        Clear all
      </Button>
    {/snippet}
  </PageHeader>

  <div class="flex gap-1" role="group" aria-label="Filter notifications">
    <Button
      size="sm"
      variant={unreadOnly ? 'ghost' : 'secondary'}
      aria-pressed={!unreadOnly}
      onclick={() => (unreadOnly = false)}
    >
      All
    </Button>
    <Button
      size="sm"
      variant={unreadOnly ? 'secondary' : 'ghost'}
      aria-pressed={unreadOnly}
      onclick={() => (unreadOnly = true)}
    >
      Unread ({unread})
    </Button>
  </div>

  {#if !visible.length}
    <PageEmpty
      title={unreadOnly ? 'Nothing unread' : 'No notifications yet'}
      description={unreadOnly
        ? "You're caught up."
        : 'Finished agents, approval requests and pipeline results show up here.'}
      icon={Bell}
    />
  {:else}
    <ul class="divide-y rounded-lg border">
      {#each visible as item (item.id)}
        <NotificationRow
          {item}
          session={sessionsQuery.data?.find((s) => s.id === item.runId)}
          project={projectsQuery.data?.find((p) => p.id === item.projectId)}
          onOpen={open}
          onMarkRead={(id) => notificationInbox.markRead(id)}
          onRemove={(id) => notificationInbox.remove(id)}
        />
      {/each}
    </ul>
  {/if}
</div>
