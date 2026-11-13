<script lang="ts">
  import AISessionCard from '$lib/domains/ai/components/shared/AISessionCard.svelte';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import { buildReference } from '$lib/domains/shared/utils/entityReference';
  import { copyText } from '$lib/domains/shared/utils/clipboard';
  import { openCoderSession } from '../utils/launch';
  import { buildCoderThreadMenu } from '../utils/threadMenu';
  import type { CoderThread } from '../types.js';

  interface Props {
    thread: CoderThread;
    onClick?: () => void;
    onDelete?: (thread: CoderThread) => void;
    isActive?: boolean;
    isRunning?: boolean;
    queuedCount?: number;
    subAgentRunning?: number;
    hideProject?: boolean;
  }

  let {
    thread,
    onClick,
    onDelete,
    isActive = false,
    isRunning = false,
    queuedCount = 0,
    subAgentRunning = 0,
    hideProject = false,
  }: Props = $props();

  const messageCount = $derived(
    thread.message_count ??
      thread.messages.filter((m) => m.role === 'user' || m.role === 'assistant').length
  );

  const inlineBadges = $derived(
    thread.thread_kind === 'coordinator'
      ? [{ label: 'Coordinator', variant: 'secondary' as const }]
      : []
  );

  const trailingBadges = $derived.by(() => {
    const badges: {
      label: string;
      variant: 'outline';
      class?: string;
    }[] = [];
    if (subAgentRunning > 0) {
      badges.push({
        label: `${subAgentRunning} agent${subAgentRunning === 1 ? '' : 's'}`,
        variant: 'outline',
        class: 'border-purple-500/30 text-purple-600 dark:text-purple-400',
      });
    }
    if (isRunning) {
      badges.push({
        label: 'Active',
        variant: 'outline',
        class: 'border-primary/30 text-primary',
      });
    }
    return badges;
  });
</script>

<ContextMenuList
  items={buildCoderThreadMenu(thread, {
    onOpen: onClick ? () => onClick() : undefined,
    onCopyReference: () =>
      copyText(
        buildReference({
          kind: 'coder session',
          id: thread.id,
          title: thread.title,
          fields: { Workspace: thread.workspace_root },
        }),
        'Session reference copied'
      ),
    onLaunchWorkspace: thread.workspace_root
      ? () => openCoderSession({ workspaceRoot: thread.workspace_root })
      : undefined,
    onDelete,
  })}
>
  {#snippet trigger(menuProps)}
    <AISessionCard
      {...menuProps}
      title={thread.title}
      {isActive}
      {isRunning}
      {queuedCount}
      updatedAt={thread.updated_at}
      {messageCount}
      subtitle={hideProject ? null : thread.workspace_root || null}
      {inlineBadges}
      {trailingBadges}
      deleteTitle="Delete session"
      {onClick}
      onDelete={onDelete ? () => onDelete(thread) : undefined}
    />
  {/snippet}
</ContextMenuList>
