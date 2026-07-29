<script lang="ts">
  import AISessionCard from "$lib/domains/ai/components/shared/AISessionCard.svelte";
  import type { Conversation } from "../../types/index.js";

  interface Props {
    conversation: Conversation;
    onClick?: () => void;
    onDelete?: (conversation: Conversation) => void;
    isActive?: boolean;
  }

  let { conversation, onClick, onDelete, isActive = false }: Props = $props();

  // Provider is the same for every thread, so only the model is worth showing.
  const subtitle = $derived(conversation.model ?? conversation.provider);
</script>

<AISessionCard
  title={conversation.title}
  {isActive}
  updatedAt={conversation.updated_at}
  messageCount={conversation.message_count ?? 0}
  subtitle={subtitle || null}
  deleteTitle="Delete conversation"
  {onClick}
  onDelete={onDelete ? () => onDelete(conversation) : undefined}
/>
