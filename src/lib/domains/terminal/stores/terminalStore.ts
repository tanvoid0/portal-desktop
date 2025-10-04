/**
 * Terminal Store
 * Centralized state management for terminal tabs and processes
 */

import { writable, derived } from 'svelte/store';
import type { TerminalState, TerminalTab, TerminalProcess, TerminalOutput, TerminalSettings } from '../types';

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
  defaultShell: typeof window !== 'undefined' && navigator.userAgent.includes('Windows') ? 'powershell.exe' : 'bash',
  workingDirectory: typeof window !== 'undefined' && navigator.userAgent.includes('Windows') ? 'C:\\' : '/'
};

// Initial state
const initialState: TerminalState = {
  tabs: [],
  processes: [],
  activeTabId: null,
  settings: defaultSettings,
  isLoading: false,
  error: null,
  output: {}
};

// Create the main store
export const terminalStore = writable<TerminalState>(initialState);

// Derived stores
export const activeTab = derived(terminalStore, ($store) => {
  if (!$store.activeTabId) return null;
  return $store.tabs.find(tab => tab.id === $store.activeTabId) || null;
});

export const activeProcess = derived(terminalStore, ($store) => {
  const tab = $store.tabs.find(tab => tab.id === $store.activeTabId);
  if (!tab?.processId) return null;
  return $store.processes.find(process => process.id === tab.processId) || null;
});

export const activeOutput = derived(terminalStore, ($store) => {
  const process = $store.processes.find(p => p.tabId === $store.activeTabId);
  if (!process) return [];
  return $store.output[process.id] || [];
});

// Store actions
export const terminalActions = {
  // Tab management
  createTab: (name: string, workingDirectory: string, projectId?: string) => {
    const newTab: TerminalTab = {
      id: crypto.randomUUID(),
      name,
      projectId,
      workingDirectory,
      status: 'idle',
      isActive: false
    };

    terminalStore.update(state => ({
      ...state,
      tabs: [...state.tabs, newTab],
      activeTabId: newTab.id
    }));

    return newTab.id;
  },

  setActiveTab: (tabId: string) => {
    terminalStore.update(state => ({
      ...state,
      tabs: state.tabs.map(tab => ({
        ...tab,
        isActive: tab.id === tabId
      })),
      activeTabId: tabId
    }));
  },

  closeTab: (tabId: string) => {
    terminalStore.update(state => {
      const newTabs = state.tabs.filter(tab => tab.id !== tabId);
      const newProcesses = state.processes.filter(process => process.tabId !== tabId);
      
      // Clean up output for processes in this tab
      const processesToRemove = state.processes.filter(p => p.tabId === tabId);
      const newOutput = { ...state.output };
      processesToRemove.forEach(process => {
        delete newOutput[process.id];
      });

      // Set new active tab if needed
      let newActiveTabId = state.activeTabId;
      if (state.activeTabId === tabId) {
        newActiveTabId = newTabs.length > 0 ? newTabs[0].id : null;
      }

      return {
        ...state,
        tabs: newTabs,
        processes: newProcesses,
        activeTabId: newActiveTabId,
        output: newOutput
      };
    });
  },

  // Process management
  createProcess: (tabId: string, command: string, workingDirectory: string, environment: Record<string, string> = {}) => {
    const newProcess: TerminalProcess = {
      id: crypto.randomUUID(),
      tabId,
      command,
      status: 'running',
      startTime: new Date(),
      workingDirectory,
      environment
    };

    terminalStore.update(state => ({
      ...state,
      processes: [...state.processes, newProcess],
      tabs: state.tabs.map(tab => 
        tab.id === tabId 
          ? { ...tab, processId: newProcess.id, status: 'running' }
          : tab
      ),
      output: {
        ...state.output,
        [newProcess.id]: []
      }
    }));

    return newProcess.id;
  },

  updateProcessStatus: (processId: string, status: TerminalProcess['status'], exitCode?: number) => {
    terminalStore.update(state => {
      const updatedProcesses = state.processes.map(process => 
        process.id === processId 
          ? { 
              ...process, 
              status, 
              exitCode,
              endTime: status !== 'running' ? new Date() : process.endTime
            }
          : process
      );

      // Update tab status based on process
      const process = updatedProcesses.find(p => p.id === processId);
      const updatedTabs = state.tabs.map(tab => 
        tab.processId === processId 
          ? { ...tab, status: process?.status || 'idle' }
          : tab
      );

      return {
        ...state,
        processes: updatedProcesses,
        tabs: updatedTabs
      };
    });
  },

  // Output management
  addOutput: (processId: string, output: TerminalOutput) => {
    terminalStore.update(state => ({
      ...state,
      output: {
        ...state.output,
        [processId]: [...(state.output[processId] || []), output]
      }
    }));
  },

  clearOutput: (processId: string) => {
    terminalStore.update(state => ({
      ...state,
      output: {
        ...state.output,
        [processId]: []
      }
    }));
  },

  // Settings
  updateSettings: (settings: Partial<TerminalSettings>) => {
    terminalStore.update(state => ({
      ...state,
      settings: { ...state.settings, ...settings }
    }));
  },

  // Loading and error states
  setLoading: (loading: boolean) => {
    terminalStore.update(state => ({ ...state, isLoading: loading }));
  },

  setError: (error: string | null) => {
    terminalStore.update(state => ({ ...state, error }));
  }
};
