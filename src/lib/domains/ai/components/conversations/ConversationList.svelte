<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Plus, MessageCircle, Trash2 } from "@lucide/svelte";
  import AISessionSidebar from "$lib/domains/ai/components/shared/AISessionSidebar.svelte";
  import ConversationCard from "./ConversationCard.svelte";
  import type { Conversation } from "../../types/index.js";

  interface Props {
    conversations: Conversation[];
    onConversationClick?: (conversation: Conversation) => void;
    onCreateNew?: () => void;
    onDeleteConversation?: (conversation: Conversation) => void;
    onDeleteAll?: () => void;
    selectedConversationId?: string | null;
  }

  let {
    conversations = $bindable<Conversation[]>([]),
    onConversationClick,
    onCreateNew,
    onDeleteConversation,
    onDeleteAll,
    selectedConversationId,
  }: Props = $props();

  let searchQuery = $state("");
  const filteredConversations = $derived(
    conversations.filter((conv) =>
      conv.title.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );
  const isEmpty = $derived(filteredConversations.length === 0);
</script>

<AISessionSidebar
  searchPlaceholder="Search conversations..."
  bind:searchValue={searchQuery}
  {isEmpty}
>
  {#snippet toolbar()}
    <Button onclick={onCreateNew} size="sm" class="h-8 shrink-0" title="New conversation">
      <Plus class="h-3.5 w-3.5" />
    </Button>
    {#if onDeleteAll && conversations.length > 0}
      <Button
        onclick={onDeleteAll}
        size="sm"
        variant="ghost"
        class="h-8 text-destructive hover:text-destructive"
        title="Delete all conversations"
      >
        <Trash2 class="h-3.5 w-3.5" />
      </Button>
    {/if}
  {/snippet}

  {#snippet empty()}
    <div class="flex flex-1 flex-col items-center justify-center py-8 text-center text-muted-foreground">
      <MessageCircle class="mx-auto mb-1.5 h-8 w-8 opacity-50" />
      <p class="text-xs">
        {searchQuery ? "No conversations found" : "No conversations yet"}
      </p>
    </div>
  {/snippet}

  {#snippet children()}
    {#each filteredConversations as conversation (conversation.id)}
      <ConversationCard
        {conversation}
        onClick={() => onConversationClick?.(conversation)}
        onDelete={onDeleteConversation}
        isActive={selectedConversationId === conversation.id}
      />
    {/each}
  {/snippet}
</AISessionSidebar>
