<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Undo2, Check, X } from "@lucide/svelte";
  import type { ChatMessage } from "../types.js";

  interface Props {
    message: ChatMessage;
    messageIndex: number;
    canEdit?: boolean;
    onEdit?: (messageIndex: number, content: string) => void;
  }

  let {
    message,
    messageIndex,
    canEdit = false,
    onEdit,
  }: Props = $props();

  let editing = $state(false);
  let draft = $state("");

  function startEdit() {
    if (!canEdit || !onEdit) return;
    draft = message.content ?? "";
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    draft = "";
  }

  function saveEdit() {
    const trimmed = draft.trim();
    if (!trimmed || !onEdit) return;
    onEdit(messageIndex, trimmed);
    editing = false;
    draft = "";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      cancelEdit();
      return;
    }
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      saveEdit();
    }
  }
</script>

<div class="group relative rounded-md border border-border/40 bg-muted/10">
  <div class="flex items-start justify-between gap-2 px-3 py-2">
    {#if editing}
      <div class="min-w-0 flex-1 space-y-2">
        <Textarea
          bind:value={draft}
          rows={3}
          onkeydown={handleKeydown}
          class="min-h-[60px] resize-y text-xs leading-relaxed"
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
          <Button
            type="button"
            size="sm"
            variant="ghost"
            class="h-7 gap-1"
            onclick={cancelEdit}
          >
            <X class="h-3.5 w-3.5" />
            Cancel
          </Button>
          <span class="text-[11px] text-muted-foreground">Ctrl+Enter to save</span>
        </div>
      </div>
    {:else}
      <p class="min-w-0 flex-1 whitespace-pre-wrap text-xs leading-relaxed text-foreground">
        {message.content}
      </p>
      {#if canEdit && onEdit}
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          class="h-7 w-7 shrink-0 opacity-60 transition-opacity sm:opacity-0 sm:group-hover:opacity-100"
          title="Edit message and rerun from here"
          onclick={startEdit}
        >
          <Undo2 class="h-3.5 w-3.5" />
        </Button>
      {/if}
    {/if}
  </div>
</div>
