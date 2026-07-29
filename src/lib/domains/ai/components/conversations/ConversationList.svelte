<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Plus,
    MessageCircle,
    Trash2,
    FolderPlus,
    ChevronRight,
    X,
  } from "@lucide/svelte";
  import AISessionSidebar from "$lib/domains/ai/components/shared/AISessionSidebar.svelte";
  import ConversationCard from "./ConversationCard.svelte";
  import {
    addFolder,
    assignFolder,
    groupByFolder,
    loadChatFolders,
    removeFolder,
    saveChatFolders,
    type ChatFolders,
  } from "../../utils/chatFolders.js";
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
  let folders = $state<ChatFolders>(loadChatFolders());
  let newFolderName = $state<string | null>(null);
  /** Folder under the pointer mid-drag, for the drop hint. */
  let dropTarget = $state<string | null | undefined>(undefined);

  const filteredConversations = $derived(
    conversations.filter((conv) =>
      conv.title.toLowerCase().includes(searchQuery.toLowerCase()),
    ),
  );
  const groups = $derived(groupByFolder(filteredConversations, folders));
  // Folders still need to render (and accept drops) when nothing matches.
  const isEmpty = $derived(
    filteredConversations.length === 0 && folders.names.length === 0,
  );

  $effect(() => {
    saveChatFolders($state.snapshot(folders));
  });

  function commitFolder() {
    if (newFolderName) folders = addFolder(folders, newFolderName);
    newFolderName = null;
  }

  function handleDrop(event: DragEvent, name: string | null) {
    event.preventDefault();
    dropTarget = undefined;
    const id = event.dataTransfer?.getData("text/plain");
    if (id) folders = assignFolder(folders, id, name);
  }

  function handleDragOver(event: DragEvent, name: string | null) {
    event.preventDefault();
    dropTarget = name;
  }
</script>

{#snippet row(conversation: Conversation)}
  <div
    draggable="true"
    role="presentation"
    ondragstart={(e) => e.dataTransfer?.setData("text/plain", conversation.id)}
  >
    <ConversationCard
      {conversation}
      onClick={() => onConversationClick?.(conversation)}
      onDelete={onDeleteConversation}
      isActive={selectedConversationId === conversation.id}
    />
  </div>
{/snippet}

<AISessionSidebar
  title="Chats"
  searchPlaceholder="Search conversations..."
  bind:searchValue={searchQuery}
  listClass="space-y-0.5 p-1.5"
  {isEmpty}
>
  {#snippet toolbar()}
    <Button
      onclick={() => (newFolderName = "")}
      size="icon"
      variant="ghost"
      class="h-7 w-7 shrink-0"
      title="New folder"
    >
      <FolderPlus class="h-4 w-4" />
    </Button>
    <Button
      onclick={onCreateNew}
      size="icon"
      variant="ghost"
      class="h-7 w-7 shrink-0"
      title="New conversation"
    >
      <Plus class="h-4 w-4" />
    </Button>
    {#if onDeleteAll && conversations.length > 0}
      <Button
        onclick={onDeleteAll}
        size="icon"
        variant="ghost"
        class="h-7 w-7 shrink-0 text-muted-foreground hover:text-destructive"
        title="Delete all conversations"
      >
        <Trash2 class="h-4 w-4" />
      </Button>
    {/if}
  {/snippet}

  {#snippet meta()}
    {#if newFolderName !== null}
      <Input
        bind:value={newFolderName}
        placeholder="Folder name, press ⏎"
        class="h-7 text-xs"
        autofocus
        onblur={commitFolder}
        onkeydown={(e) => {
          if (e.key === "Enter") commitFolder();
          if (e.key === "Escape") newFolderName = null;
        }}
      />
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
    <!-- Drag a chat onto a folder to file it; drop on the loose list to unfile. -->
    {#each groups as group (group.name ?? "__ungrouped")}
      {#if group.name !== null}
        {@const folderName = group.name}
        <details
          class="group/folder rounded-md {dropTarget === folderName
            ? 'bg-primary/10 ring-1 ring-primary/40'
            : ''}"
          open
          ondragover={(e) => handleDragOver(e, folderName)}
          ondragleave={() => (dropTarget = undefined)}
          ondrop={(e) => handleDrop(e, folderName)}
        >
          <summary
            class="flex cursor-pointer list-none items-center gap-1 rounded-md px-2 py-1.5 text-xs text-muted-foreground marker:content-none hover:bg-muted/60 hover:text-foreground"
          >
            <ChevronRight
              class="h-3.5 w-3.5 shrink-0 transition-transform group-open/folder:rotate-90"
            />
            <span class="min-w-0 flex-1 truncate">{folderName}</span>
            <Button
              size="icon"
              variant="ghost"
              class="hidden h-5 w-5 shrink-0 hover:text-destructive group-hover/folder:flex"
              title="Delete folder (keeps its chats)"
              onclick={(e) => {
                e.preventDefault();
                folders = removeFolder(folders, folderName);
              }}
            >
              <X class="h-3 w-3" />
            </Button>
          </summary>
          <div class="space-y-0.5 pl-3">
            {#each group.items as conversation (conversation.id)}
              {@render row(conversation)}
            {/each}
            {#if group.items.length === 0}
              <p class="px-2 py-1 text-[11px] text-muted-foreground">
                Folder is empty
              </p>
            {/if}
          </div>
        </details>
      {:else}
        <div
          class="min-h-8 space-y-0.5 rounded-md {dropTarget === null
            ? 'bg-primary/10 ring-1 ring-primary/40'
            : ''}"
          role="presentation"
          ondragover={(e) => handleDragOver(e, null)}
          ondragleave={() => (dropTarget = undefined)}
          ondrop={(e) => handleDrop(e, null)}
        >
          {#each group.items as conversation (conversation.id)}
            {@render row(conversation)}
          {/each}
        </div>
      {/if}
    {/each}
  {/snippet}
</AISessionSidebar>
