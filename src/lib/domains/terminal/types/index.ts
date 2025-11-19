/**
 * Terminal Domain Types
 */

export interface TerminalTab {
  id: string;
  name: string;
  projectId?: string;
  workingDirectory: string;
  shell?: string; // Terminal shell/profile for this tab
  status: 'idle' | 'running' | 'completed' | 'error' | 'killed';
  processId?: string;
  startTime?: Date;
  endTime?: Date;
  exitCode?: number;
  isActive: boolean;
}

export interface TerminalProcess {
  id: string;
  tab_id: string; // snake_case to match Rust backend
  command: string;
  working_directory: string; // snake_case to match Rust backend
  environment: Record<string, string>;
  status: string; // string to match Rust backend
  pid?: number;
  start_time: string; // snake_case to match Rust backend
  end_time?: string; // snake_case to match Rust backend
  exit_code?: number; // snake_case to match Rust backend
}

export interface TerminalOutput {
  process_id: string; // snake_case to match Rust backend
  output_type: string; // snake_case to match Rust backend
  content: string;
  timestamp: string; // string to match Rust backend
}

export interface TerminalConfig {
  theme: 'dark' | 'light' | 'auto';
  fontSize: number;
  fontFamily: string;
  cursorStyle: 'block' | 'underline' | 'bar';
  scrollbackLines: number;
  bellSound: boolean;
  autoClose: boolean;
  confirmClose: boolean;
  defaultShell: string;
  workingDirectory: string;
}

export interface TerminalCommand {
  id: string;
  processId: string;
  command: string;
  timestamp: Date;
  status: 'pending' | 'running' | 'completed' | 'failed';
  output?: string;
  error?: string;
}

export interface TerminalState {
  tabs: TerminalTab[];
  processes: TerminalProcess[];
  activeTabId: string | null;
  settings: TerminalConfig;
  isLoading: boolean;
  error: string | null;
  output: Record<string, TerminalOutput[]>; // processId -> outputs
}

export interface TerminalContext {
  tabId: string;
  processId?: string;
  workingDirectory: string;
  environment: Record<string, string>;
}

export interface CreateProcessRequest {
  tab_id: string; // snake_case to match Rust backend
  shell: string;
  working_directory: string; // snake_case to match Rust backend
  environment: Record<string, string>;
  cols: number;
  rows: number;
}

export interface TerminalSystemInfo {
  os: string;
  arch: string;
  platform: string;
  workingDirectory: string;
  availableShells: string[];
  terminalProfiles: TerminalProfile[];
}

export interface TerminalProfile {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  icon?: string;
  description?: string;
}

export interface ShellIntegrationEvent {
  type: 'CommandStart' | 'CommandOutput' | 'CommandEnd' | 'CommandDetected';
  payload: CommandStartEvent | CommandOutputEvent | CommandEndEvent | CommandDetectedEvent;
}

export interface CommandStartEvent {
  id: string;
  command: string;
  workingDirectory: string;
  timestamp: Date;
}

export interface CommandOutputEvent {
  process_id: string;
  content: string;
  timestamp: Date;
}

export interface CommandEndEvent {
  id: string;
  exitCode: number;
  duration: number;
  timestamp: Date;
}

export interface CommandDetectedEvent {
  command: string;
  workingDirectory: string;
  timestamp: Date;
}

export interface TerminalOutputEvent {
  process_id: string;
  content: string;
  timestamp: Date;
}