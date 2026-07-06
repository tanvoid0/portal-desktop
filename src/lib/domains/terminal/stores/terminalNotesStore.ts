import { writable } from "svelte/store";
import { invokeClient } from "$lib/utils/invokeClient";

type NotesByTabId = Record<string, string>;

function createTerminalNotesStore() {
  const { subscribe, update } = writable<NotesByTabId>({});

  let cache: NotesByTabId = {};
  const loadedTabs = new Set<string>();

  // Keep a local cache for synchronous get.
  subscribe((value) => {
    cache = value;
  });

  return {
    subscribe,

    getNote: (tabId: string): string => cache[tabId] ?? "",

    loadNote: async (tabId: string): Promise<void> => {
      if (loadedTabs.has(tabId)) return;
      try {
        const markdown = await invokeClient.request<string>(
          "load_terminal_note",
          { data: { tabId } },
        );

        update((state) => ({
          ...state,
          [tabId]: markdown || "",
        }));
      } catch (e) {
        // If no row exists yet, treat as empty note.
        update((state) => ({
          ...state,
          [tabId]: "",
        }));
      } finally {
        loadedTabs.add(tabId);
      }
    },

    saveNote: async (tabId: string, markdown: string): Promise<void> => {
      // Always upsert local cache first for instant UI response.
      update((state) => ({
        ...state,
        [tabId]: markdown,
      }));

      try {
        await invokeClient.request("save_terminal_note", {
          data: { tabId, markdown },
        });
        loadedTabs.add(tabId);
      } catch (e) {
        // Keep local cache; user can retry on next input.
        update((state) => ({
          ...state,
          [tabId]: state[tabId] ?? markdown,
        }));
      }
    },
  };
}

export const terminalNotesStore = createTerminalNotesStore();

