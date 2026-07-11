<script lang="ts">
  import AISessionCard from "$lib/domains/ai/components/shared/AISessionCard.svelte";
  import type { Conversation } from "../../types/index.js";

  interface Props {
    conversation: Conversation;
    onClick?: () => void;
    onDelete?: (conversation: Conversation) => void;
    isActive?: boolean;
    compact?: boolean;
  }

  let {
    conversation,
    onClick,
    onDelete,
    isActive = false,
    compact = false,
  }: Props = $props();

  const subtitle = $derived(
    [conversation.provider, conversation.model].filter(Boolean).join(" · "),
  );
</script>

<AISessionCard
  title={conversation.title}
  {isActive}
  {compact}
  updatedAt={conversation.updated_at}
  messageCount={conversation.message_count ?? 0}
  subtitle={subtitle || null}
  deleteTitle="Delete conversation"
  onclick={onClick}
  onDelete={onDelete ? () => onDelete(conversation) : undefined}
/>
