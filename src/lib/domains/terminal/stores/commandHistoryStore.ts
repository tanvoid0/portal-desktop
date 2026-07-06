/**
 * Command History Store
 * Manages command history with input/output tracking
 */

import { writable } from "svelte/store";
import { cleanTerminalOutput } from "../utils/textUtils";
import { invokeClient } from "$lib/utils/invokeClient";

export interface CommandHistoryEntry {
  id: string;
  tabId: string;
  timestamp: Date;
  command: string;
  output: string;
  exitCode?: number;
  duration?: number;
  intercepted?: boolean;
}

export interface CommandHistoryState {
  entries: Record<string, CommandHistoryEntry[]>; // tabId -> entries
  maxEntries: number;
  searchQuery: string;
  filteredEntries: Record<string, CommandHistoryEntry[]>; // tabId -> filtered entries
}

const initialState: CommandHistoryState = {
  entries: {},
  maxEntries: 100,
  searchQuery: "",
  filteredEntries: {},
};

function createCommandHistoryStore() {
  const { subscribe, set, update } =
    writable<CommandHistoryState>(initialState);

  return {
    subscribe,

    addEntry: (
      tabId: string,
      entry: Omit<CommandHistoryEntry, "id" | "timestamp" | "tabId">,
    ) => {
      update((state) => {
        const newEntry: CommandHistoryEntry = {
          ...entry,
          id: crypto.randomUUID(),
          tabId,
          timestamp: new Date(),
          output: cleanTerminalOutput(entry.output), // Clean the output before storing
        };

        const tabEntries = state.entries[tabId] || [];
        const newTabEntries = [newEntry, ...tabEntries].slice(
          0,
          state.maxEntries,
        );

        return {
          ...state,
          entries: {
            ...state.entries,
            [tabId]: newTabEntries,
          },
        };
      });
    },

    clearHistory: (tabId?: string) => {
      update((state) => {
        if (tabId) {
          // Clear history for specific tab
          const newEntries = { ...state.entries };
          delete newEntries[tabId];
          return {
            ...state,
            entries: newEntries,
          };
        } else {
          // Clear all history
          return {
            ...state,
            entries: {},
          };
        }
      });
    },

    getEntry: (tabId: string, id: string) => {
      let entry: CommandHistoryEntry | undefined;
      update((state) => {
        const tabEntries = state.entries[tabId] || [];
        entry = tabEntries.find((e) => e.id === id);
        return state;
      });
      return entry;
    },

    updateEntry: (
      tabId: string,
      id: string,
      updates: Partial<CommandHistoryEntry>,
    ) => {
      update((state) => {
        const tabEntries = state.entries[tabId] || [];
        const updatedTabEntries = tabEntries.map((entry) =>
          entry.id === id ? { ...entry, ...updates } : entry,
        );

        return {
          ...state,
          entries: {
            ...state.entries,
            [tabId]: updatedTabEntries,
          },
        };
      });
    },

    getTabHistory: (tabId: string) => {
      let entries: CommandHistoryEntry[] = [];
      update((state) => {
        entries = state.entries[tabId] || [];
        return state;
      });
      return entries;
    },

    // Reactive getter for tab history
    getTabHistoryReactive: (tabId: string) => {
      return {
        subscribe: (callback: (entries: CommandHistoryEntry[]) => void) => {
          return subscribe((state) => {
            const entries = state.entries[tabId] || [];
            callback(entries);
          });
        },
      };
    },

    // Search functionality
    setSearchQuery: (query: string) => {
      update((state) => {
        const newState = { ...state, searchQuery: query };

        // Filter entries for each tab
        const filteredEntries: Record<string, CommandHistoryEntry[]> = {};
        Object.keys(state.entries).forEach((tabId) => {
          const entries = state.entries[tabId] || [];
          if (query.trim() === "") {
            filteredEntries[tabId] = entries;
          } else {
            const searchLower = query.toLowerCase();
            filteredEntries[tabId] = entries.filter(
              (entry) =>
                entry.command.toLowerCase().includes(searchLower) ||
                entry.output.toLowerCase().includes(searchLower),
            );
          }
        });

        return {
          ...newState,
          filteredEntries,
        };
      });
    },

    getFilteredHistory: (tabId: string) => {
      let entries: CommandHistoryEntry[] = [];
      update((state) => {
        entries = state.filteredEntries[tabId] || state.entries[tabId] || [];
        return state;
      });
      return entries;
    },

    // Persistence methods
    saveToBackend: async (tabId: string) => {
      try {
        const entries = commandHistoryStore.getTabHistory(tabId);
        await invokeClient.request("save_command_history", {
          data: {
            tabId,
            entries: entries.map((entry) => {
              const payload: Record<string, any> = {
                id: entry.id,
                tabId,
                timestamp: entry.timestamp.toISOString(),
                command: entry.command,
                output: entry.output,
              };

              if (entry.exitCode !== undefined) {
                payload.exitCode = entry.exitCode;
              }
              if (entry.duration !== undefined) {
                payload.duration = entry.duration;
              }
              if (entry.intercepted !== undefined) {
                payload.intercepted = entry.intercepted;
              }

              return payload;
            }),
          },
        });
        console.log("Command history saved to backend for tab:", tabId);
      } catch (error) {
        console.error("Failed to save command history:", error);
      }
    },

    loadFromBackend: async (tabId: string) => {
      try {
        const rawEntries = await invokeClient.request<any[]>(
          "load_command_history",
          { data: { tabId } },
        );

        update((state) => ({
          ...state,
          entries: {
            ...state.entries,
            [tabId]: rawEntries.map((entry) => ({
              id: entry.id,
              tabId,
              timestamp: new Date(entry.timestamp),
              command: entry.command,
              output: entry.output,
              exitCode: entry.exit_code ?? undefined,
              duration: entry.duration ?? undefined,
              intercepted: entry.intercepted ?? undefined,
            })),
          },
        }));
        console.log("Command history loaded from backend for tab:", tabId);
      } catch (error) {
        console.error("Failed to load command history:", error);
      }
    },

    // Auto-save when entries are added
    addEntryWithPersistence: async (
      tabId: string,
      entry: Omit<CommandHistoryEntry, "id" | "timestamp" | "tabId">,
    ) => {
      // Add entry to store
      commandHistoryStore.addEntry(tabId, entry);

      // Save to backend
      await commandHistoryStore.saveToBackend(tabId);
    },
  };
}

export const commandHistoryStore = createCommandHistoryStore();
