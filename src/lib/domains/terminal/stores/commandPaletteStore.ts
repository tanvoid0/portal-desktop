/**
 * Command Palette Store
 * Manages command palette state and actions
 */

import { writable } from 'svelte/store';
import { commandHistoryStore, type CommandHistoryEntry } from './commandHistoryStore';

export interface CommandPaletteAction {
  id: string;
  label: string;
  description: string;
  icon: string;
  action: () => void | Promise<void>;
  keywords: string[];
}

export interface CommandPaletteState {
  isOpen: boolean;
  query: string;
  selectedIndex: number;
  actions: CommandPaletteAction[];
  filteredActions: CommandPaletteAction[];
}

const initialState: CommandPaletteState = {
  isOpen: false,
  query: '',
  selectedIndex: 0,
  actions: [],
  filteredActions: []
};

function createCommandPaletteStore() {
  const { subscribe, set, update } = writable<CommandPaletteState>(initialState);

  return {
    subscribe,
    
    open: () => {
      update(state => ({
        ...state,
        isOpen: true,
        query: '',
        selectedIndex: 0
      }));
    },
    
    close: () => {
      update(state => ({
        ...state,
        isOpen: false,
        query: '',
        selectedIndex: 0
      }));
    },
    
    setQuery: (query: string) => {
      update(state => {
        const filteredActions = state.actions.filter(action => 
          action.label.toLowerCase().includes(query.toLowerCase()) ||
          action.description.toLowerCase().includes(query.toLowerCase()) ||
          action.keywords.some(keyword => keyword.toLowerCase().includes(query.toLowerCase()))
        );
        
        return {
          ...state,
          query,
          filteredActions,
          selectedIndex: 0
        };
      });
    },
    
    setSelectedIndex: (index: number) => {
      update(state => ({
        ...state,
        selectedIndex: Math.max(0, Math.min(index, state.filteredActions.length - 1))
      }));
    },
    
    selectNext: () => {
      update(state => ({
        ...state,
        selectedIndex: Math.min(state.selectedIndex + 1, state.filteredActions.length - 1)
      }));
    },
    
    selectPrevious: () => {
      update(state => ({
        ...state,
        selectedIndex: Math.max(state.selectedIndex - 1, 0)
      }));
    },
    
    executeSelected: async () => {
      update(state => {
        const selectedAction = state.filteredActions[state.selectedIndex];
        if (selectedAction) {
          selectedAction.action();
        }
        return {
          ...state,
          isOpen: false,
          query: '',
          selectedIndex: 0
        };
      });
    },
    
    setActions: (actions: CommandPaletteAction[]) => {
      update(state => ({
        ...state,
        actions,
        filteredActions: actions
      }));
    },
    
    addAction: (action: CommandPaletteAction) => {
      update(state => ({
        ...state,
        actions: [...state.actions, action],
        filteredActions: [...state.filteredActions, action]
      }));
    },
    
    removeAction: (actionId: string) => {
      update(state => ({
        ...state,
        actions: state.actions.filter(action => action.id !== actionId),
        filteredActions: state.filteredActions.filter(action => action.id !== actionId)
      }));
    }
  };
}

export const commandPaletteStore = createCommandPaletteStore();
