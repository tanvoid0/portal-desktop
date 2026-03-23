/**
 * AI Terminal Store
 * Manages command blocks, AI interactions, and terminal state for warp-style terminal
 */

import { writable, derived, get } from 'svelte/store';
import type { TerminalProcess, TerminalOutput } from '../types';
import { parseCommandOutput } from '../utils/outputParser';

// Helper function to parse output for display
// This uses the same parser for all output, including interactive prompts
function parseOutputForDisplay(
  rawOutput: string,
  command: string,
  shellType?: 'bash' | 'zsh' | 'powershell' | 'cmd' | 'fish' | 'sh'
): string {
  if (!rawOutput) return '';
  // Use the same parser for all output - it handles interactive prompts too
  return parseCommandOutput(rawOutput, command, shellType);
}

export interface CommandBlock {
  id: string;
  command: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'paused';
  output: string; // Parsed/cleaned output for display
  rawOutput: string; // Raw terminal output for parsing
  exitCode?: number;
  startTime: Date;
  endTime?: Date;
  duration?: number;
  processId?: string;
  waitingForInput?: boolean;
  inputPrompt?: string;
  inputType?: 'password' | 'text' | 'confirm';
}

export interface TerminalTab {
  id: string;
  name: string;
  commandBlocks: CommandBlock[];
  activeBlockId: string | null;
  process: TerminalProcess | null;
  isConnected: boolean;
  createdAt: Date;
}

export interface AITerminalState {
  tabs: TerminalTab[];
  activeTabId: string;
  commandBlocks: CommandBlock[]; // Active tab's blocks (derived)
  activeBlockId: string | null; // Active tab's active block (derived)
  currentProcess: TerminalProcess | null; // Active tab's process (derived)
  isConnected: boolean; // Active tab's connection status (derived)
  viewMode: 'terminal' | 'ai-terminal' | 'ai-only';
  aiSuggestions: string[];
  pendingInput: {
    blockId: string;
    prompt: string;
    type: 'password' | 'text' | 'confirm';
  } | null;
}

function createInitialTab(): TerminalTab {
  return {
    id: crypto.randomUUID(),
    name: 'Terminal 1',
    commandBlocks: [],
    activeBlockId: null,
    process: null,
    isConnected: false,
    createdAt: new Date()
  };
}

const initialTab = createInitialTab();
const initialState: AITerminalState = {
  tabs: [initialTab],
  activeTabId: initialTab.id,
  commandBlocks: [],
  activeBlockId: null,
  currentProcess: null,
  isConnected: false,
  viewMode: 'terminal',
  aiSuggestions: [],
  pendingInput: null
};

function createAITerminalStore() {
  const { subscribe, set, update } = writable<AITerminalState>(initialState);

  // Helper to get active tab
  const getActiveTab = (state: AITerminalState): TerminalTab | null => {
    return state.tabs.find(t => t.id === state.activeTabId) || null;
  };

  // Helper to sync derived properties from active tab
  const syncActiveTab = (state: AITerminalState): AITerminalState => {
    const activeTab = getActiveTab(state);
    if (activeTab) {
      return {
        ...state,
        commandBlocks: activeTab.commandBlocks,
        activeBlockId: activeTab.activeBlockId,
        currentProcess: activeTab.process,
        isConnected: activeTab.isConnected
      };
    }
    return state;
  };

  return {
    subscribe,

    // Tab management
    addTab: (name?: string): string => {
      let tabId = '';
      update(state => {
        const tabNumber = state.tabs.length + 1;
        const newTab: TerminalTab = {
          id: crypto.randomUUID(),
          name: name || `Terminal ${tabNumber}`,
          commandBlocks: [],
          activeBlockId: null,
          process: null,
          isConnected: false,
          createdAt: new Date()
        };
        tabId = newTab.id;
        return syncActiveTab({
          ...state,
          tabs: [...state.tabs, newTab],
          activeTabId: newTab.id
        });
      });
      return tabId;
    },

    setActiveTab: (tabId: string) => {
      update(state => syncActiveTab({
        ...state,
        activeTabId: tabId
      }));
    },

    closeTab: (tabId: string) => {
      update(state => {
        const newTabs = state.tabs.filter(t => t.id !== tabId);
        if (newTabs.length === 0) {
          // Always keep at least one tab
          const newTab = createInitialTab();
          return syncActiveTab({
            ...state,
            tabs: [newTab],
            activeTabId: newTab.id
          });
        }

        let newActiveTabId = state.activeTabId;
        if (state.activeTabId === tabId) {
          // Switch to the first remaining tab
          newActiveTabId = newTabs[0].id;
        }

        return syncActiveTab({
          ...state,
          tabs: newTabs,
          activeTabId: newActiveTabId
        });
      });
    },

    renameTab: (tabId: string, name: string) => {
      update(state => ({
        ...state,
        tabs: state.tabs.map(tab =>
          tab.id === tabId ? { ...tab, name } : tab
        )
      }));
    },

    // Add a new command block
    addCommandBlock: (command: string): string => {
      const blockId = crypto.randomUUID();
      const block: CommandBlock = {
        id: blockId,
        command,
        status: 'pending',
        output: '',
        rawOutput: '',
        startTime: new Date(),
        processId: undefined
      };

      update(state => {
        const newTabs = state.tabs.map(tab =>
          tab.id === state.activeTabId
            ? {
                ...tab,
                commandBlocks: [...tab.commandBlocks, block],
                activeBlockId: blockId
              }
            : tab
        );
        return syncActiveTab({
          ...state,
          tabs: newTabs
        });
      });

      return blockId;
    },
    
    // Update command block status
    updateBlockStatus: (blockId: string, status: CommandBlock['status']) => {
      update(state => {
        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(block =>
            block.id === blockId ? { ...block, status } : block
          )
        }));
        return syncActiveTab({
          ...state,
          tabs: newTabs
        });
      });
    },
    
    // Append output to a command block (raw output)
    appendOutput: (blockId: string, content: string) => {
      update(state => {
        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(block =>
            block.id === blockId
              ? {
                  ...block,
                  rawOutput: block.rawOutput + content,
                  // Parse output in real-time for display (will be re-parsed on completion)
                  output: parseOutputForDisplay(block.rawOutput + content, block.command)
                }
              : block
          )
        }));
        return syncActiveTab({
          ...state,
          tabs: newTabs
        });
      });
    },
    
    // Parse and set the final output for a command block
    parseAndSetOutput: (blockId: string, command: string, shellType?: string) => {
      update(state => {
        // Find the block's raw output from tabs
        let rawOutput = '';
        for (const tab of state.tabs) {
          const block = tab.commandBlocks.find(b => b.id === blockId);
          if (block) {
            rawOutput = block.rawOutput;
            break;
          }
        }

        if (!rawOutput && rawOutput !== '') return state;

        const parsedOutput = parseOutputForDisplay(rawOutput, command, shellType as any);

        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(b =>
            b.id === blockId
              ? { ...b, output: parsedOutput }
              : b
          )
        }));

        return syncActiveTab({
          ...state,
          tabs: newTabs
        });
      });
    },
    
    // Complete a command block
    completeBlock: (blockId: string, exitCode?: number) => {
      update(state => {
        // Find the block to get its start time
        let startTime: Date | undefined;
        for (const tab of state.tabs) {
          const block = tab.commandBlocks.find(b => b.id === blockId);
          if (block) {
            startTime = block.startTime;
            break;
          }
        }

        if (!startTime) return state;

        const endTime = new Date();
        const duration = endTime.getTime() - startTime.getTime();

        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(b =>
            b.id === blockId
              ? {
                  ...b,
                  status: (exitCode === 0 || exitCode === undefined ? 'completed' : 'failed') as CommandBlock['status'],
                  exitCode,
                  endTime,
                  duration,
                  waitingForInput: false,
                  inputPrompt: undefined,
                  inputType: undefined
                }
              : b
          )
        }));

        return syncActiveTab({
          ...state,
          tabs: newTabs,
          pendingInput: state.pendingInput?.blockId === blockId ? null : state.pendingInput
        });
      });
    },
    
    // Set block as waiting for input
    setBlockWaitingForInput: (
      blockId: string,
      prompt: string,
      type: 'password' | 'text' | 'confirm'
    ) => {
      update(state => {
        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(block =>
            block.id === blockId
              ? {
                  ...block,
                  status: 'paused' as const,
                  waitingForInput: true,
                  inputPrompt: prompt,
                  inputType: type
                }
              : block
          )
        }));

        return syncActiveTab({
          ...state,
          tabs: newTabs,
          pendingInput: { blockId, prompt, type }
        });
      });
    },

    // Provide input to a waiting block
    provideInput: (blockId: string, input: string) => {
      update(state => {
        const newTabs = state.tabs.map(tab => ({
          ...tab,
          commandBlocks: tab.commandBlocks.map(block =>
            block.id === blockId
              ? {
                  ...block,
                  status: 'running' as const,
                  waitingForInput: false,
                  inputPrompt: undefined,
                  inputType: undefined
                }
              : block
          )
        }));

        return syncActiveTab({
          ...state,
          tabs: newTabs,
          pendingInput: state.pendingInput?.blockId === blockId ? null : state.pendingInput
        });
      });
    },
    
    // Set current process
    setCurrentProcess: (process: TerminalProcess | null) => {
      update(state => ({ ...state, currentProcess: process }));
    },
    
    // Set connection status
    setConnected: (connected: boolean) => {
      update(state => ({ ...state, isConnected: connected }));
    },
    
    // Set view mode
    setViewMode: (mode: 'terminal' | 'ai-terminal' | 'ai-only') => {
      update(state => ({ ...state, viewMode: mode }));
    },
    
    // Add AI suggestion
    addAISuggestion: (suggestion: string) => {
      update(state => ({
        ...state,
        aiSuggestions: [...state.aiSuggestions, suggestion]
      }));
    },
    
    // Clear AI suggestions
    clearAISuggestions: () => {
      update(state => ({ ...state, aiSuggestions: [] }));
    },
    
    // Get active block
    getActiveBlock: (): CommandBlock | null => {
      const state = get({ subscribe });
      return state.commandBlocks.find(b => b.id === state.activeBlockId) || null;
    },
    
    // Clear all blocks
    clearBlocks: () => {
      update(state => {
        const newTabs = state.tabs.map(tab =>
          tab.id === state.activeTabId
            ? {
                ...tab,
                commandBlocks: [],
                activeBlockId: null
              }
            : tab
        );

        return syncActiveTab({
          ...state,
          tabs: newTabs,
          pendingInput: null
        });
      });
    },
    
    // Reset store
    reset: () => {
      set(initialState);
    }
  };
}

export const aiTerminalStore = createAITerminalStore();

// Derived stores
export const activeCommandBlock = derived(aiTerminalStore, $store =>
  $store.commandBlocks.find(b => b.id === $store.activeBlockId) || null
);

export const runningBlocks = derived(aiTerminalStore, $store =>
  $store.commandBlocks.filter(b => b.status === 'running' || b.status === 'paused')
);

export const completedBlocks = derived(aiTerminalStore, $store =>
  $store.commandBlocks.filter(b => b.status === 'completed' || b.status === 'failed')
);

