/**
 * Command History Store
 * Manages command history with input/output tracking
 */

import { writable } from 'svelte/store';
import { cleanTerminalOutput } from '../utils/textUtils';

export interface CommandHistoryEntry {
  id: string;
  timestamp: Date;
  command: string;
  output: string;
  exitCode?: number;
  duration?: number;
  intercepted?: boolean;
}

export interface CommandHistoryState {
  entries: CommandHistoryEntry[];
  maxEntries: number;
}

const initialState: CommandHistoryState = {
  entries: [],
  maxEntries: 100
};

function createCommandHistoryStore() {
  const { subscribe, set, update } = writable<CommandHistoryState>(initialState);

  return {
    subscribe,
    
    addEntry: (entry: Omit<CommandHistoryEntry, 'id' | 'timestamp'>) => {
      update(state => {
        const newEntry: CommandHistoryEntry = {
          ...entry,
          id: crypto.randomUUID(),
          timestamp: new Date(),
          output: cleanTerminalOutput(entry.output) // Clean the output before storing
        };
        
        const newEntries = [newEntry, ...state.entries].slice(0, state.maxEntries);
        
        return {
          ...state,
          entries: newEntries
        };
      });
    },
    
    clearHistory: () => {
      update(state => ({
        ...state,
        entries: []
      }));
    },
    
    getEntry: (id: string) => {
      let entry: CommandHistoryEntry | undefined;
      update(state => {
        entry = state.entries.find(e => e.id === id);
        return state;
      });
      return entry;
    },
    
    updateEntry: (id: string, updates: Partial<CommandHistoryEntry>) => {
      update(state => ({
        ...state,
        entries: state.entries.map(entry => 
          entry.id === id ? { ...entry, ...updates } : entry
        )
      }));
    }
  };
}

export const commandHistoryStore = createCommandHistoryStore();
