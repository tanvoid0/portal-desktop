/**
 * Terminal Store
 * Unified state management for terminal tabs and processes
 */

import { writable, derived } from 'svelte/store';
import type { TerminalSettings } from '../types/index';

// Storage keys
const STORAGE_KEY = 'portal-terminal-state';
const STORAGE_VERSION = '1.0';

// Persistence functions
function saveStateToStorage(state: TerminalState): void {
  if (typeof window === 'undefined') return;
  
  try {
    const serializableState = {
      version: STORAGE_VERSION,
      tabs: state.tabs.map(tab => ({
        ...tab,
        createdAt: tab.createdAt.toISOString(),
        lastActivity: tab.lastActivity?.toISOString()
      })),
      processes: state.processes.map(process => ({
        ...process,
        startTime: process.startTime.toISOString(),
        endTime: process.endTime?.toISOString()
      })),
      activeTabId: state.activeTabId,
      settings: state.settings
    };
    
    localStorage.setItem(STORAGE_KEY, JSON.stringify(serializableState));
  } catch (error) {
    console.warn('Failed to save terminal state to localStorage:', error);
  }
}

function loadStateFromStorage(): TerminalState {
  if (typeof window === 'undefined') return initialState;
  
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return initialState;
    
    const parsed = JSON.parse(stored);
    
    // Check version compatibility
    if (parsed.version !== STORAGE_VERSION) {
      console.warn('Terminal state version mismatch, using default state');
      return initialState;
    }
    
    // Restore state with proper date objects
    const restoredState: TerminalState = {
      tabs: parsed.tabs.map((tab: any) => ({
        ...tab,
        createdAt: new Date(tab.createdAt),
        lastActivity: tab.lastActivity ? new Date(tab.lastActivity) : undefined,
        status: 'inactive' // Reset all tabs to inactive on restore
      })),
      processes: [], // Clear processes since backend restarts kill them
      activeTabId: parsed.activeTabId,
      settings: parsed.settings || defaultSettings,
      isConnected: false, // Always start disconnected
      isLoading: false,
      error: null
    };
    
    console.log('Restored terminal state from localStorage:', restoredState);
    return restoredState;
  } catch (error) {
    console.warn('Failed to load terminal state from localStorage:', error);
    return initialState;
  }
}

export interface TerminalTab {
  id: string;
  title: string;
  processId?: string; // Associated terminal process ID
  type: 'terminal' | 'editor' | 'file' | 'custom';
  closable?: boolean;
  icon?: string;
  status: 'active' | 'inactive' | 'loading' | 'error' | 'disconnected';
  workingDirectory: string;
  shell?: string;
  createdAt: Date;
  lastActivity?: Date;
  // Resource context for project-specific terminals
  resourceName?: string; // e.g., 'project', 'workspace', 'container'
  resourceId?: string;   // e.g., project ID, workspace ID, container ID
  // Terminal content persistence
  savedOutput?: string;  // Saved terminal output for restoration
  currentWorkingDirectory?: string; // Current working directory
}

export interface TerminalProcess {
  id: string;
  tabId: string;
  command: string;
  workingDirectory: string;
  environment: Record<string, string>;
  status: 'running' | 'stopped' | 'error';
  startTime: Date;
  endTime?: Date;
  exitCode?: number;
  pid?: number;
}

interface TerminalState {
  tabs: TerminalTab[];
  processes: TerminalProcess[];
  activeTabId: string | null;
  settings: TerminalSettings;
  isConnected: boolean;
  isLoading: boolean;
  error: string | null;
}

// Default settings
const defaultSettings: TerminalSettings = {
  theme: 'dark',
  fontSize: 14,
  fontFamily: 'Monaco, Consolas, "Courier New", monospace',
  cursorStyle: 'block',
  scrollbackLines: 1000,
  bellSound: false,
  autoClose: true,
  confirmClose: true,
  defaultShell: typeof window !== 'undefined' && navigator.userAgent.includes('Windows') ? 'cmd.exe' : 'zsh',
  workingDirectory: typeof window !== 'undefined' && navigator.userAgent.includes('Windows') ? 'C:\\' : '/home/tan'
};

// Initial state
const initialState: TerminalState = {
  tabs: [],
  processes: [],
  activeTabId: null,
  settings: defaultSettings,
  isConnected: false,
  isLoading: false,
  error: null
};

// Create the main store with persistence
export const terminalStore = writable<TerminalState>(loadStateFromStorage());

// Auto-save state changes to localStorage
terminalStore.subscribe(saveStateToStorage);

// Derived stores
export const activeTab = derived(terminalStore, ($store) => {
  if (!$store.activeTabId) return null;
  return $store.tabs.find(tab => tab.id === $store.activeTabId) || null;
});

export const activeProcess = derived(terminalStore, ($store) => {
  const activeTab = $store.tabs.find(tab => tab.id === $store.activeTabId);
  if (!activeTab?.processId) return null;
  return $store.processes.find(process => process.id === activeTab.processId) || null;
});

export const tabCount = derived(terminalStore, ($store) => $store.tabs.length);

export const runningProcesses = derived(terminalStore, ($store) => 
  $store.processes.filter(process => process.status === 'running')
);

// Store actions
export const terminalActions = {
  // Tab management
  createTab: (tab: Omit<TerminalTab, 'id' | 'createdAt' | 'status'>) => {
    console.log('TerminalStore: Creating new tab:', tab);
    const newTab: TerminalTab = {
      id: crypto.randomUUID(),
      status: 'inactive',
      createdAt: new Date(),
      ...tab
    };

    console.log('TerminalStore: New tab object:', newTab);

    terminalStore.update(state => {
      console.log('TerminalStore: Current state before update:', state);
      const updatedTabs = state.tabs.map(t => ({ ...t, status: 'inactive' as const }));
      const newState = {
        ...state,
        tabs: [...updatedTabs, newTab],
        activeTabId: newTab.id
      };
      console.log('TerminalStore: New state after update:', newState);
      return newState;
    });

    // Set the new tab as active
    terminalActions.setActiveTab(newTab.id);
    console.log('TerminalStore: Created tab with ID:', newTab.id);
    return newTab.id;
  },

  setActiveTab: (tabId: string) => {
    terminalStore.update(state => ({
      ...state,
      activeTabId: tabId,
      tabs: state.tabs.map(tab => ({
        ...tab,
        status: tab.id === tabId ? 'active' : 'inactive',
        lastActivity: tab.id === tabId ? new Date() : tab.lastActivity
      }))
    }));
  },

  closeTab: (tabId: string) => {
    terminalStore.update(state => {
      const tabToClose = state.tabs.find(tab => tab.id === tabId);
      if (!tabToClose) return state;

      // Kill associated process if running
      const processToKill = state.processes.find(process => process.tabId === tabId);
      if (processToKill && processToKill.status === 'running') {
        // Kill the process in the background
        import('../services/terminalService').then(({ TerminalService }) => {
          TerminalService.killProcess(processToKill.id).catch(console.error);
        });
      }

      // Remove the process from the store
      const newProcesses = state.processes.filter(process => process.tabId !== tabId);

      const newTabs = state.tabs.filter(tab => tab.id !== tabId);
      let newActiveTabId = state.activeTabId;

      // If we're closing the active tab, switch to another tab
      if (state.activeTabId === tabId) {
        newActiveTabId = newTabs.length > 0 ? newTabs[0].id : null;
        if (newActiveTabId) {
          newTabs.forEach(tab => {
            if (tab.id === newActiveTabId) {
              tab.status = 'active';
            }
          });
        }
      }

      return {
        ...state,
        tabs: newTabs,
        processes: newProcesses,
        activeTabId: newActiveTabId
      };
    });
  },

  updateTab: (tabId: string, updates: Partial<TerminalTab>) => {
    terminalStore.update(state => ({
      ...state,
      tabs: state.tabs.map(tab =>
        tab.id === tabId ? { ...tab, ...updates } : tab
      )
    }));
  },

  // Process management
  createProcess: (process: Omit<TerminalProcess, 'id' | 'startTime'>) => {
    const newProcess: TerminalProcess = {
      id: crypto.randomUUID(),
      startTime: new Date(),
      ...process
    };

    terminalStore.update(state => ({
      ...state,
      processes: [...state.processes, newProcess],
      tabs: state.tabs.map(tab =>
        tab.id === process.tabId
          ? { ...tab, processId: newProcess.id, status: 'active' }
          : tab
      )
    }));

    return newProcess.id;
  },

  updateProcess: (processId: string, updates: Partial<TerminalProcess>) => {
    terminalStore.update(state => ({
      ...state,
      processes: state.processes.map(process =>
        process.id === processId ? { ...process, ...updates } : process
      )
    }));
  },

  stopProcess: (processId: string) => {
    terminalStore.update(state => ({
      ...state,
      processes: state.processes.map(process =>
        process.id === processId
          ? { ...process, status: 'stopped', endTime: new Date() }
          : process
      ),
      tabs: state.tabs.map(tab =>
        tab.processId === processId
          ? { ...tab, status: 'disconnected' }
          : tab
      )
    }));
  },

  // Connection management
  setConnected: (connected: boolean) => {
    terminalStore.update(state => ({ ...state, isConnected: connected }));
  },

  setLoading: (loading: boolean) => {
    terminalStore.update(state => ({ ...state, isLoading: loading }));
  },

  setError: (error: string | null) => {
    terminalStore.update(state => ({ ...state, error }));
  },

  // Settings
  updateSettings: (settings: Partial<TerminalSettings>) => {
    terminalStore.update(state => ({
      ...state,
      settings: { ...state.settings, ...settings }
    }));
  },

  // Utility functions
  getTabByProcessId: (processId: string) => {
    let result: TerminalTab | null = null;
    terminalStore.subscribe(state => {
      result = state.tabs.find(tab => tab.processId === processId) || null;
    })();
    return result;
  },

  getProcessByTabId: (tabId: string) => {
    let result: TerminalProcess | null = null;
    terminalStore.subscribe(state => {
      const tab = state.tabs.find(tab => tab.id === tabId);
      if (tab?.processId) {
        result = state.processes.find(process => process.id === tab.processId) || null;
      }
    })();
    return result;
  },

  // Cleanup
  clearAll: () => {
    terminalStore.set(initialState);
  },

  clearStorage: () => {
    if (typeof window !== 'undefined') {
      localStorage.removeItem(STORAGE_KEY);
    }
    terminalStore.set(initialState);
  },

  // Save terminal output for a tab
  saveTerminalOutput: (tabId: string, output: string, workingDirectory?: string) => {
    terminalStore.update(state => ({
      ...state,
      tabs: state.tabs.map(tab =>
        tab.id === tabId
          ? { 
              ...tab, 
              savedOutput: output,
              currentWorkingDirectory: workingDirectory || tab.currentWorkingDirectory,
              lastActivity: new Date()
            }
          : tab
      )
    }));
  },

  // Get saved terminal output for a tab
  getSavedTerminalOutput: (tabId: string) => {
    let result: { output: string; workingDirectory?: string } | null = null;
    terminalStore.subscribe(state => {
      const tab = state.tabs.find(tab => tab.id === tabId);
      if (tab) {
        result = {
          output: tab.savedOutput || '',
          workingDirectory: tab.currentWorkingDirectory
        };
      }
    })();
    return result;
  },

  // Clean up stale data (tabs older than 24 hours)
  cleanupStaleData: () => {
    const now = new Date();
    const staleThreshold = 24 * 60 * 60 * 1000; // 24 hours in milliseconds
    
    terminalStore.update(state => {
      const staleTabs = state.tabs.filter(tab => {
        const age = now.getTime() - tab.createdAt.getTime();
        return age > staleThreshold;
      });
      
      if (staleTabs.length === 0) return state;
      
      console.log(`Cleaning up ${staleTabs.length} stale terminal tabs`);
      
      const remainingTabs = state.tabs.filter(tab => {
        const age = now.getTime() - tab.createdAt.getTime();
        return age <= staleThreshold;
      });
      
      // Remove processes for stale tabs
      const remainingProcesses = state.processes.filter(process => 
        !staleTabs.some(tab => tab.id === process.tabId)
      );
      
      // Update active tab if it was stale
      let newActiveTabId = state.activeTabId;
      if (staleTabs.some(tab => tab.id === state.activeTabId)) {
        newActiveTabId = remainingTabs.length > 0 ? remainingTabs[0].id : null;
        if (newActiveTabId) {
          remainingTabs.forEach(tab => {
            if (tab.id === newActiveTabId) {
              tab.status = 'active';
            }
          });
        }
      }
      
      return {
        ...state,
        tabs: remainingTabs,
        processes: remainingProcesses,
        activeTabId: newActiveTabId
      };
    });
  },

  // Resource-specific terminal management
  getTabsForResource: (resourceName: string, resourceId: string) => {
    let result: TerminalTab[] = [];
    terminalStore.subscribe(state => {
      result = state.tabs.filter(tab => 
        tab.resourceName === resourceName && tab.resourceId === resourceId
      );
    })();
    return result;
  },

  getTabsForResourceName: (resourceName: string) => {
    let result: TerminalTab[] = [];
    terminalStore.subscribe(state => {
      result = state.tabs.filter(tab => tab.resourceName === resourceName);
    })();
    return result;
  },

  getGlobalTabs: () => {
    let result: TerminalTab[] = [];
    terminalStore.subscribe(state => {
      result = state.tabs.filter(tab => !tab.resourceName && !tab.resourceId);
    })();
    return result;
  },

  closeTabsForResource: (resourceName: string, resourceId: string) => {
    terminalStore.update(state => {
      const tabsToClose = state.tabs.filter(tab => 
        tab.resourceName === resourceName && tab.resourceId === resourceId
      );
      
      const remainingTabs = state.tabs.filter(tab => 
        !(tab.resourceName === resourceName && tab.resourceId === resourceId)
      );

      // Stop associated processes
      const newProcesses = state.processes.map(process => 
        tabsToClose.some(tab => tab.id === process.tabId) && process.status === 'running'
          ? { ...process, status: 'stopped' as const, endTime: new Date() }
          : process
      );

      // Update active tab if needed
      let newActiveTabId = state.activeTabId;
      if (tabsToClose.some(tab => tab.id === state.activeTabId)) {
        newActiveTabId = remainingTabs.length > 0 ? remainingTabs[0].id : null;
        if (newActiveTabId) {
          remainingTabs.forEach(tab => {
            if (tab.id === newActiveTabId) {
              tab.status = 'active';
            }
          });
        }
      }

      return {
        ...state,
        tabs: remainingTabs,
        processes: newProcesses,
        activeTabId: newActiveTabId
      };
    });
  }
};

// Theme sync will be handled by components that use both stores