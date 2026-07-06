<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import ConversationCard from "./ConversationCard.svelte";
  import { Plus, MessageCircle, Trash2 } from "@lucide/svelte";
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
  let filteredConversations = $derived(
    conversations.filter((conv) =>
      conv.title.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );
</script>

<div class="flex h-full flex-col">
  <div class="flex items-center gap-2 border-b p-2.5">
    <Input
      placeholder="Search conversations..."
      bind:value={searchQuery}
      class="h-8 flex-1 text-sm"
    />
    <Button
      onclick={onCreateNew}
      size="sm"
      class="h-8"
      title="New Conversation"
    >
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
  </div>
  <ScrollArea class="flex-1">
    <div class="space-y-2 p-2">
      {#if filteredConversations.length === 0}
        <div class="py-6 text-center text-muted-foreground">
          <MessageCircle class="mx-auto mb-1.5 h-8 w-8 opacity-50" />
          <p class="text-xs">
            {searchQuery ? "No conversations found" : "No conversations yet"}
          </p>
        </div>
      {:else}
        {#each filteredConversations as conversation}
          <ConversationCard
            {conversation}
            onClick={() => onConversationClick?.(conversation)}
            onDelete={onDeleteConversation}
            isActive={selectedConversationId === conversation.id}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>
</div>
