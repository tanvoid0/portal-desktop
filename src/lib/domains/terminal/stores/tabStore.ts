/**
 * Tab Store
 * Manages terminal tabs with reordering and duplication
 */

import { writable } from 'svelte/store';
import { sessionStore } from './sessionStore';

export interface TerminalTab {
  id: string;
  title: string;
  workingDirectory: string;
  shell: string;
  isActive: boolean;
  hasRunningProcess: boolean;
  lastActivity: string;
  processId?: string;
}

export interface TabState {
  tabs: TerminalTab[];
  activeTabId: string | null;
  nextTabId: number;
}

const initialState: TabState = {
  tabs: [],
  activeTabId: null,
  nextTabId: 1
};

function createTabStore() {
  const { subscribe, set, update } = writable<TabState>(initialState);

  return {
    subscribe,
    
    addTab: (tab: Omit<TerminalTab, 'id' | 'isActive' | 'hasRunningProcess' | 'lastActivity'>) => {
      const newTab: TerminalTab = {
        ...tab,
        id: `tab-${Date.now()}`,
        isActive: true,
        hasRunningProcess: false,
        lastActivity: new Date().toISOString()
      };
      
      update(state => {
        // Deactivate all other tabs
        const updatedTabs = state.tabs.map(t => ({ ...t, isActive: false }));
        
        return {
          ...state,
          tabs: [...updatedTabs, newTab],
          activeTabId: newTab.id
        };
      });
      
      return newTab.id;
    },
    
    removeTab: (tabId: string) => {
      update(state => {
        const newTabs = state.tabs.filter(tab => tab.id !== tabId);
        const wasActive = state.activeTabId === tabId;
        
        let newActiveTabId = state.activeTabId;
        if (wasActive && newTabs.length > 0) {
          // Activate the next tab or the last tab
          const currentIndex = state.tabs.findIndex(tab => tab.id === tabId);
          const nextIndex = currentIndex < newTabs.length ? currentIndex : newTabs.length - 1;
          newActiveTabId = newTabs[nextIndex].id;
        } else if (newTabs.length === 0) {
          newActiveTabId = null;
        }
        
        return {
          ...state,
          tabs: newTabs,
          activeTabId: newActiveTabId
        };
      });
    },
    
    activateTab: (tabId: string) => {
      update(state => ({
        ...state,
        tabs: state.tabs.map(tab => ({
          ...tab,
          isActive: tab.id === tabId
        })),
        activeTabId: tabId
      }));
    },
    
    duplicateTab: (tabId: string) => {
      update(state => {
        const tabToDuplicate = state.tabs.find(tab => tab.id === tabId);
        if (!tabToDuplicate) return state;
        
        const duplicatedTab: TerminalTab = {
          ...tabToDuplicate,
          id: `tab-${Date.now()}`,
          isActive: true,
          hasRunningProcess: false,
          lastActivity: new Date().toISOString()
        };
        
        // Deactivate all other tabs
        const updatedTabs = state.tabs.map(tab => ({ ...tab, isActive: false }));
        
        return {
          ...state,
          tabs: [...updatedTabs, duplicatedTab],
          activeTabId: duplicatedTab.id
        };
      });
    },
    
    reorderTabs: (fromIndex: number, toIndex: number) => {
      update(state => {
        const newTabs = [...state.tabs];
        const [movedTab] = newTabs.splice(fromIndex, 1);
        newTabs.splice(toIndex, 0, movedTab);
        
        return {
          ...state,
          tabs: newTabs
        };
      });
    },
    
    updateTab: (tabId: string, updates: Partial<TerminalTab>) => {
      update(state => ({
        ...state,
        tabs: state.tabs.map(tab => 
          tab.id === tabId ? { ...tab, ...updates } : tab
        )
      }));
    },
    
    setTabRunning: (tabId: string, isRunning: boolean, processId?: string) => {
      update(state => ({
        ...state,
        tabs: state.tabs.map(tab => 
          tab.id === tabId 
            ? { 
                ...tab, 
                hasRunningProcess: isRunning,
                processId: isRunning ? processId : undefined,
                lastActivity: new Date().toISOString()
              } 
            : tab
        )
      }));
    },
    
    getActiveTab: (): TerminalTab | null => {
      let activeTab: TerminalTab | null = null;
      update(state => {
        activeTab = state.tabs.find(tab => tab.isActive) || null;
        return state;
      });
      return activeTab;
    },
    
    getTab: (tabId: string): TerminalTab | null => {
      let tab: TerminalTab | null = null;
      update(state => {
        tab = state.tabs.find(t => t.id === tabId) || null;
        return state;
      });
      return tab;
    },
    
    clearAllTabs: () => {
      update(state => ({
        ...state,
        tabs: [],
        activeTabId: null
      }));
    }
  };
}

export const tabStore = createTabStore();
