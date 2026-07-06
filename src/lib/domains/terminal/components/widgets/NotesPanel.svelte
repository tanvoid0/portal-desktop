<script lang="ts">
  import { onMount } from "svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import { terminalNotesStore } from "../../stores/terminalNotesStore";
  import { isTauriEnvironment } from "$lib/utils/tauri";
  import { logger } from "$lib/domains/shared";

  const log = logger.createScoped("NotesPanel");

  interface Props {
    tabId: string;
  }

  let { tabId }: Props = $props();

  let noteMarkdown = $state("");
  let notesSaveTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  const isTauri = isTauriEnvironment();

  onMount(async () => {
    if (!isTauri) return;
    await terminalNotesStore.loadNote(tabId);
    noteMarkdown = terminalNotesStore.getNote(tabId);
  });

  function scheduleNotesSave(markdown: string) {
    if (!isTauri) return;
    if (notesSaveTimer) clearTimeout(notesSaveTimer);
    notesSaveTimer = setTimeout(async () => {
      try {
        await terminalNotesStore.saveNote(tabId, markdown);
      } catch (e) {
        log.warn("Failed to save notes", { e });
      }
    }, 800);
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-hidden">
  <div class="border-b border-gray-700 p-2">
    <div class="text-sm font-medium text-gray-200">Notes</div>
    <div class="text-xs text-gray-400">
      Tab: <span class="font-mono">{tabId.slice(0, 8)}...</span>
    </div>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto p-3">
    <Textarea
      class="h-full min-h-48 resize-none border-gray-700 bg-gray-900 font-mono text-xs text-gray-200"
      bind:value={noteMarkdown}
      placeholder="Add notes for this terminal tab (markdown supported)..."
      oninput={(e) => {
        const value = (e.target as HTMLTextAreaElement).value;
        noteMarkdown = value;
        scheduleNotesSave(value);
      }}
    />
    <div class="mt-2 text-xs text-gray-500">
      {isTauri ? "Saved automatically (debounced)." : "Notes are disabled in browser mode."}
    </div>
  </div>
</div>
