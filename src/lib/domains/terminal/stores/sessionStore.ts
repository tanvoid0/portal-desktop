/**
 * Session Store
 * Manages terminal session state and persistence
 */

import { writable } from "svelte/store";
import { invokeClient } from "$lib/utils/invokeClient";

export interface TerminalSession {
  tab_id: string;
  working_directory: string;
  environment: Record<string, string>;
  scrollback_buffer: string[];
  cursor_position: [number, number];
  terminal_size: [number, number];
  last_activity: string;
  process_id?: string;
}

export interface SessionState {
  sessions: Record<string, TerminalSession>;
  activeSession: string | null;
  maxScrollbackLines: number;
}

const initialState: SessionState = {
  sessions: {},
  activeSession: null,
  maxScrollbackLines: 10000,
};

function fromInvokeSession(raw: Record<string, unknown>): TerminalSession {
  return {
    tab_id: String(raw.tab_id ?? raw.tabId ?? ""),
    working_directory: String(
      raw.working_directory ?? raw.workingDirectory ?? "",
    ),
    environment: (raw.environment as Record<string, string>) ?? {},
    scrollback_buffer: (raw.scrollback_buffer ??
      raw.scrollbackBuffer ??
      []) as string[],
    cursor_position: (raw.cursor_position ??
      raw.cursorPosition ??
      [0, 0]) as [number, number],
    terminal_size: (raw.terminal_size ??
      raw.terminalSize ??
      [80, 24]) as [number, number],
    last_activity: String(raw.last_activity ?? raw.lastActivity ?? ""),
    process_id: (raw.process_id ?? raw.processId) as string | undefined,
  };
}

function createSessionStore() {
  const { subscribe, set, update } = writable<SessionState>(initialState);

  return {
    subscribe,

    saveSession: async (session: TerminalSession) => {
      try {
        await invokeClient.request("save_terminal_session", {
          data: { session },
        });
        update((state) => ({
          ...state,
          sessions: {
            ...state.sessions,
            [session.tab_id]: session,
          },
        }));
        console.log("Session saved for tab:", session.tab_id);
      } catch (error) {
        console.error("Failed to save session:", error);
      }
    },

    loadSession: async (tabId: string): Promise<TerminalSession | null> => {
      try {
        const session = await invokeClient.request<Record<string, unknown> | null>(
          "load_terminal_session",
          { data: { tabId } },
        );
        if (session) {
          const normalized = fromInvokeSession(session);
          update((state) => ({
            ...state,
            sessions: {
              ...state.sessions,
              [tabId]: normalized,
            },
            activeSession: tabId,
          }));
          console.log("Session loaded for tab:", tabId);
          return normalized;
        }
        return null;
      } catch (error) {
        console.error("Failed to load session:", error);
        return null;
      }
    },

    deleteSession: async (tabId: string) => {
      try {
        await invokeClient.request("delete_terminal_session", {
          data: { tabId },
        });
        update((state) => {
          const newSessions = { ...state.sessions };
          delete newSessions[tabId];
          return {
            ...state,
            sessions: newSessions,
            activeSession:
              state.activeSession === tabId ? null : state.activeSession,
          };
        });
        console.log("Session deleted for tab:", tabId);
      } catch (error) {
        console.error("Failed to delete session:", error);
      }
    },

    listSessions: async (): Promise<string[]> => {
      try {
        const tabIds = await invokeClient.request<string[]>(
          "list_terminal_sessions",
        );
        return tabIds;
      } catch (error) {
        console.error("Failed to list sessions:", error);
        return [];
      }
    },

    clearAllSessions: async () => {
      try {
        await invokeClient.request("clear_all_sessions");
        update((state) => ({
          ...state,
          sessions: {},
          activeSession: null,
        }));
        console.log("All sessions cleared");
      } catch (error) {
        console.error("Failed to clear all sessions:", error);
      }
    },

    setActiveSession: (tabId: string | null) => {
      update((state) => ({
        ...state,
        activeSession: tabId,
      }));
    },

    updateSession: (tabId: string, updates: Partial<TerminalSession>) => {
      update((state) => {
        const currentSession = state.sessions[tabId];
        if (currentSession) {
          const updatedSession = { ...currentSession, ...updates };
          return {
            ...state,
            sessions: {
              ...state.sessions,
              [tabId]: updatedSession,
            },
          };
        }
        return state;
      });
    },

    getSession: (tabId: string): TerminalSession | null => {
      let session: TerminalSession | null = null;
      update((state) => {
        session = state.sessions[tabId] || null;
        return state;
      });
      return session;
    },

    setMaxScrollbackLines: (maxLines: number) => {
      update((state) => ({
        ...state,
        maxScrollbackLines: maxLines,
      }));
    },
  };
}

export const sessionStore = createSessionStore();
