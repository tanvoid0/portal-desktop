<script lang="ts">
  import { FolderOpen } from "@lucide/svelte";
  import AISessionCard from "$lib/domains/ai/components/shared/AISessionCard.svelte";
  import type { CoderThread } from "../types.js";

  interface Props {
    thread: CoderThread;
    onClick?: () => void;
    onDelete?: (thread: CoderThread) => void;
    isActive?: boolean;
    isRunning?: boolean;
    queuedCount?: number;
    subAgentRunning?: number;
    hideProject?: boolean;
    compact?: boolean;
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
    compact = false,
  }: Props = $props();

  const messageCount = $derived(
    thread.message_count ??
      thread.messages.filter((m) => m.role === "user" || m.role === "assistant")
        .length,
  );

  const workspaceName = $derived(
    (thread.workspace_root ?? "")
      .split(/[/\\]/)
      .filter(Boolean)
      .pop() ?? thread.workspace_root ?? "",
  );

  const inlineBadges = $derived(
    thread.thread_kind === "coordinator"
      ? [{ label: "Coordinator", variant: "secondary" as const }]
      : [],
  );

  const trailingBadges = $derived.by(() => {
    const badges: {
      label: string;
      variant: "outline";
      class?: string;
    }[] = [];
    if (subAgentRunning > 0) {
      badges.push({
        label: `${subAgentRunning} agent${subAgentRunning === 1 ? "" : "s"}`,
        variant: "outline",
        class: "border-purple-500/30 text-purple-600 dark:text-purple-400",
      });
    }
    if (isRunning) {
      badges.push({
        label: "Active",
        variant: "outline",
        class: "border-primary/30 text-primary",
      });
    }
    return badges;
  });
</script>

<AISessionCard
  title={thread.title}
  {isActive}
  {isRunning}
  {compact}
  {queuedCount}
  updatedAt={thread.updated_at}
  messageCount={messageCount}
  subtitle={workspaceName || null}
  subtitleTitle={thread.workspace_root}
  subtitleIcon={FolderOpen}
  hideSubtitle={hideProject || !workspaceName}
  {inlineBadges}
  {trailingBadges}
  deleteTitle="Delete session"
  {onClick}
  onDelete={onDelete ? () => onDelete(thread) : undefined}
/>
