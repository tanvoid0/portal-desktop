/**
 * Terminal Domain Exports
 */

// Types
export type * from "./types";

// Config
export { defaultTerminalConfig } from "./config/defaultTerminalConfig";

// Services
export { TerminalService } from "./services/terminalService";
export {
  CommandInterceptionService,
  type InputPrompt,
  type InterceptionResult,
} from "./services/commandInterceptionService";

// Stores
export {
  terminalStore,
  terminalActions,
  activeTab,
  activeProcess,
  tabCount,
  runningProcesses,
} from "./stores/terminalStore";
export {
  commandHistoryStore,
  type CommandHistoryEntry,
} from "./stores/commandHistoryStore";
export {
  commandBlockStore,
  type CapturedCommand,
  type CommandBlockSource,
} from "./stores/commandBlockStore";

// Composables
export { XtermSession } from "./composables/useXtermSession";
export {
  createTerminalProcess,
  subscribeProcessOutput,
  sendProcessInput,
  killTerminalProcess,
  getProcessExitCode,
} from "./composables/useTerminalProcess";
export { resolveXtermTheme } from "./theme";
export { disposeTerminalOutputBus } from "./services/terminalOutputBus";

// Components
export { default as CommandHistory } from "./components/CommandHistory.svelte";
export { default as TabBar } from "./components/TabBar.svelte";
export { default as TabContainer } from "./components/TabContainer.svelte";
export { default as TerminalWorkspace } from "./components/TerminalWorkspace.svelte";
export { default as ProjectTerminal } from "./components/ProjectTerminal.svelte";
export { default as Terminal } from "./components/core/Terminal.svelte";
export { default as TerminalSession } from "./components/core/TerminalSession.svelte";
export { default as CommandPalette } from "./components/CommandPalette.svelte";
export { default as CommandInput } from "./components/ai/CommandInput.svelte";
export { default as BlocksView } from "./components/core/BlocksView.svelte";
export { default as CommandBlock } from "./components/CommandBlock.svelte";
export { default as AiResponse } from "./components/ai/AiResponse.svelte";

// AI context helpers
export {
  buildTerminalContext,
  buildExplainPrompt,
  resolveShellMetadata,
  parseAiResponse,
  type AiResponseSegment,
} from "./services/terminalAiContext";
