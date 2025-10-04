/**
 * Terminal Process Manager
 * Full control over terminal processes, commands, and output
 */

import { invoke } from '@tauri-apps/api/core';
import type { TerminalProcess, TerminalOutput, TerminalCommand } from '../types';

export interface ProcessConfig {
  shell: string;
  workingDirectory: string;
  environment: Record<string, string>;
  cols: number;
  rows: number;
}

export interface CommandInterceptor {
  pattern: RegExp;
  handler: (command: string, process: TerminalProcess) => Promise<boolean>; // Return true to intercept
}

export interface OutputParser {
  pattern: RegExp;
  handler: (output: string, process: TerminalProcess) => void;
}

export class TerminalProcessManager {
  private static instance: TerminalProcessManager;
  private processes = new Map<string, TerminalProcess>();
  private commandInterceptors: CommandInterceptor[] = [];
  private outputParsers: OutputParser[] = [];
  private outputCallbacks = new Map<string, (output: TerminalOutput) => void>();

  static getInstance(): TerminalProcessManager {
    if (!TerminalProcessManager.instance) {
      TerminalProcessManager.instance = new TerminalProcessManager();
    }
    return TerminalProcessManager.instance;
  }

  /**
   * Create a new terminal process with full control
   */
  async createProcess(
    tabId: string,
    config: ProcessConfig
  ): Promise<TerminalProcess> {
    try {
      const process = await invoke<TerminalProcess>('create_terminal_process', {
        tabId,
        config
      });

      this.processes.set(process.id, process);
      return process;
    } catch (error) {
      console.error('Failed to create terminal process:', error);
      throw error;
    }
  }

  /**
   * Send input to a process with command interception
   */
  async sendInput(processId: string, input: string): Promise<void> {
    const process = this.processes.get(processId);
    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }

    // Check for command interception
    const intercepted = await this.interceptCommand(input, process);
    if (intercepted) {
      return; // Command was intercepted, don't send to process
    }

    // Send to actual process
    await invoke('send_terminal_input', { processId, input });
  }

  /**
   * Kill a process with full cleanup
   */
  async killProcess(processId: string): Promise<void> {
    try {
      await invoke('kill_terminal_process', { processId });
      this.processes.delete(processId);
      this.outputCallbacks.delete(processId);
    } catch (error) {
      console.error('Failed to kill process:', error);
      throw error;
    }
  }

  /**
   * Get process information
   */
  getProcess(processId: string): TerminalProcess | undefined {
    return this.processes.get(processId);
  }

  /**
   * Get all processes
   */
  getAllProcesses(): TerminalProcess[] {
    return Array.from(this.processes.values());
  }

  /**
   * Subscribe to process output with full control
   */
  async subscribeToOutput(
    processId: string,
    callback: (output: TerminalOutput) => void
  ): Promise<() => void> {
    this.outputCallbacks.set(processId, callback);

    // Subscribe to backend output
    const unsubscribe = await invoke<() => void>('subscribe_terminal_output', {
      processId,
      callback: (output: TerminalOutput) => {
        // Parse output before calling callback
        this.parseOutput(output, processId);
        callback(output);
      }
    });

    return () => {
      this.outputCallbacks.delete(processId);
      unsubscribe();
    };
  }

  /**
   * Add command interceptor for full control
   */
  addCommandInterceptor(interceptor: CommandInterceptor): void {
    this.commandInterceptors.push(interceptor);
  }

  /**
   * Remove command interceptor
   */
  removeCommandInterceptor(pattern: RegExp): void {
    this.commandInterceptors = this.commandInterceptors.filter(
      i => i.pattern.toString() !== pattern.toString()
    );
  }

  /**
   * Add output parser for full control
   */
  addOutputParser(parser: OutputParser): void {
    this.outputParsers.push(parser);
  }

  /**
   * Remove output parser
   */
  removeOutputParser(pattern: RegExp): void {
    this.outputParsers = this.outputParsers.filter(
      p => p.pattern.toString() !== pattern.toString()
    );
  }

  /**
   * Execute command with full control and interception
   */
  async executeCommand(
    processId: string,
    command: string
  ): Promise<TerminalCommand> {
    const process = this.processes.get(processId);
    if (!process) {
      throw new Error(`Process ${processId} not found`);
    }

    // Check for interception
    const intercepted = await this.interceptCommand(command, process);
    if (intercepted) {
      return {
        id: crypto.randomUUID(),
        processId,
        command,
        timestamp: new Date(),
        status: 'completed',
        output: 'Command intercepted and handled'
      };
    }

    // Execute normally
    return await invoke<TerminalCommand>('execute_terminal_command', {
      processId,
      command
    });
  }

  /**
   * Get system information
   */
  async getSystemInfo(): Promise<{
    platform: string;
    shell: string;
    workingDirectory: string;
    availableShells: string[];
  }> {
    return await invoke('get_system_info');
  }

  /**
   * Resize terminal
   */
  async resizeTerminal(processId: string, cols: number, rows: number): Promise<void> {
    await invoke('resize_terminal', { processId, cols, rows });
  }

  /**
   * Get process output history
   */
  async getOutputHistory(processId: string): Promise<TerminalOutput[]> {
    return await invoke<TerminalOutput[]>('get_terminal_output_history', {
      processId
    });
  }

  /**
   * Clear process output history
   */
  async clearOutputHistory(processId: string): Promise<void> {
    await invoke('clear_terminal_output_history', { processId });
  }

  /**
   * Private: Intercept commands before execution
   */
  private async interceptCommand(
    command: string,
    process: TerminalProcess
  ): Promise<boolean> {
    for (const interceptor of this.commandInterceptors) {
      if (interceptor.pattern.test(command)) {
        const intercepted = await interceptor.handler(command, process);
        if (intercepted) {
          return true;
        }
      }
    }
    return false;
  }

  /**
   * Private: Parse output for custom handling
   */
  private parseOutput(output: TerminalOutput, processId: string): void {
    const process = this.processes.get(processId);
    if (!process) return;

    for (const parser of this.outputParsers) {
      if (parser.pattern.test(output.content)) {
        parser.handler(output.content, process);
      }
    }
  }
}

// Export singleton instance
export const terminalProcessManager = TerminalProcessManager.getInstance();
