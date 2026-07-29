<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import ChatUserBubble from '$lib/domains/ai/components/chat/ChatUserBubble.svelte';
  import { Undo2, Check, X } from '@lucide/svelte';
  import type { ChatMessage } from '../types.js';

  interface Props {
    message: ChatMessage;
    messageIndex: number;
    canEdit?: boolean;
    onEdit?: (messageIndex: number, content: string) => void;
  }

  let { message, messageIndex, canEdit = false, onEdit }: Props = $props();

  let editing = $state(false);
  let draft = $state('');

  function startEdit() {
    if (!canEdit || !onEdit) return;
    draft = message.content ?? '';
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    draft = '';
  }

  function saveEdit() {
    const trimmed = draft.trim();
    if (!trimmed || !onEdit) return;
    onEdit(messageIndex, trimmed);
    editing = false;
    draft = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      cancelEdit();
      return;
    }
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      saveEdit();
    }
  }
</script>

<div class="group flex items-start justify-end gap-1.5">
  {#if canEdit && onEdit && !editing}
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      class="mt-1 h-7 w-7 shrink-0 opacity-60 transition-opacity sm:opacity-0 sm:group-hover:opacity-100"
      title="Edit message and rerun from here"
      onclick={startEdit}
    >
      <Undo2 class="h-3.5 w-3.5" />
    </Button>
  {/if}

  <ChatUserBubble density="compact" class={editing ? 'w-full max-w-[85%]' : ''}>
    {#if !editing}
      <p class="whitespace-pre-wrap">{message.content}</p>
    {:else}
      <div class="space-y-2">
        <Textarea
          bind:value={draft}
          rows={3}
          onkeydown={handleKeydown}
          class="min-h-[60px] resize-y bg-background text-xs leading-relaxed"
          placeholder="Edit your message…"
        />
        <div class="flex items-center gap-2">
          <Button
            type="button"
            size="sm"
            class="h-7 gap-1"
            disabled={!draft.trim()}
            onclick={saveEdit}
          >
            <Check class="h-3.5 w-3.5" />
            Save & rerun
          </Button>
          <Button type="button" size="sm" variant="ghost" class="h-7 gap-1" onclick={cancelEdit}>
            <X class="h-3.5 w-3.5" />
            Cancel
          </Button>
          <span class="text-[11px] text-muted-foreground">Ctrl+Enter to save</span>
        </div>
      </div>
    {/if}
  </ChatUserBubble>
</div>
