/**
 * Terminal Service - Domain-Specific Backend Integration
 * High-level API for terminal operations with full Tauri backend integration
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { TerminalProcess, TerminalOutput, TerminalCommand, TerminalContext, CreateProcessRequest } from '../types';
import type { CommandInterceptor, OutputParser } from './terminalProcessManager';

export interface ExecuteCommandRequest {
  command: string;
  workingDirectory?: string;
  environment?: Record<string, string>;
}

export class TerminalService {
  private static commandInterceptors: CommandInterceptor[] = [];
  private static outputParsers: OutputParser[] = [];
  private static outputCallbacks = new Map<string, (output: TerminalOutput) => void>();

  /**
   * Create a new terminal process with full control
   */
  static async createProcess(tabId: string, config: Partial<CreateProcessRequest> = {}): Promise<TerminalProcess> {
    const defaultConfig: CreateProcessRequest = {
      shell: navigator.userAgent.includes('Windows') ? 'powershell.exe' : 'bash',
      working_directory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/',
      environment: {},
      cols: 80,
      rows: 24,
      ...config
    };

    try {
      const process = await invoke<TerminalProcess>('create_terminal_process', {
        request: defaultConfig
      });

      // Set up output listener for this process
      this.setupOutputListener(process.id);

      return process;
    } catch (error) {
      console.error('Failed to create terminal process:', error);
      throw error;
    }
  }

  /**
   * Send input with command interception
   */
  static async sendInput(processId: string, input: string): Promise<void> {
    try {
      // Check for command interception
      const intercepted = await this.checkCommandInterception(processId, input);
      if (intercepted) {
        return;
      }

      await invoke('send_terminal_input', {
        processId,
        input
      });
    } catch (error) {
      console.error('Failed to send input:', error);
      throw error;
    }
  }

  /**
   * Kill process with full cleanup
   */
  static async killProcess(processId: string): Promise<void> {
    try {
      await invoke('kill_terminal_process', { processId });
    } catch (error) {
      console.error('Failed to kill process:', error);
      throw error;
    }
  }

  /**
   * Get process information
   */
  static async getProcess(processId: string): Promise<TerminalProcess | null> {
    try {
      return await invoke<TerminalProcess | null>('get_terminal_process', { processId });
    } catch (error) {
      console.error('Failed to get process:', error);
      return null;
    }
  }

  /**
   * Get all processes
   */
  static async getAllProcesses(): Promise<TerminalProcess[]> {
    try {
      return await invoke<TerminalProcess[]>('get_terminal_processes');
    } catch (error) {
      console.error('Failed to get processes:', error);
      return [];
    }
  }

  /**
   * Subscribe to output with full control
   */
  static async subscribeToOutput(
    processId: string,
    callback: (output: TerminalOutput) => void
  ): Promise<() => void> {
    this.outputCallbacks.set(processId, callback);
    
    return () => {
      this.outputCallbacks.delete(processId);
    };
  }

  /**
   * Execute command with interception
   */
  static async executeCommand(command: string, workingDirectory?: string): Promise<string> {
    try {
      const request: ExecuteCommandRequest = {
        command,
        workingDirectory,
        environment: {}
      };

      return await invoke<string>('execute_command', { request });
    } catch (error) {
      console.error('Failed to execute command:', error);
      throw error;
    }
  }

  /**
   * Add command interceptor for full control
   */
  static async addCommandInterceptor(interceptor: CommandInterceptor): Promise<void> {
    try {
      await invoke('add_command_interceptor', {
        interceptor: {
          pattern: interceptor.pattern.source,
          handler_type: 'monitor' // Default to monitor
        }
      });
      this.commandInterceptors.push(interceptor);
    } catch (error) {
      console.error('Failed to add command interceptor:', error);
      throw error;
    }
  }

  /**
   * Remove command interceptor
   */
  static async removeCommandInterceptor(pattern: RegExp): Promise<void> {
    try {
      await invoke('remove_command_interceptor', { pattern: pattern.source });
      this.commandInterceptors = this.commandInterceptors.filter(i => i.pattern !== pattern);
    } catch (error) {
      console.error('Failed to remove command interceptor:', error);
      throw error;
    }
  }

  /**
   * Add output parser for full control
   */
  static async addOutputParser(parser: OutputParser): Promise<void> {
    try {
      await invoke('add_output_parser', {
        parser: {
          pattern: parser.pattern.source,
          parser_type: 'highlight' // Default to highlight
        }
      });
      this.outputParsers.push(parser);
    } catch (error) {
      console.error('Failed to add output parser:', error);
      throw error;
    }
  }

  /**
   * Remove output parser
   */
  static async removeOutputParser(pattern: RegExp): Promise<void> {
    try {
      await invoke('remove_output_parser', { pattern: pattern.source });
      this.outputParsers = this.outputParsers.filter(p => p.pattern !== pattern);
    } catch (error) {
      console.error('Failed to remove output parser:', error);
      throw error;
    }
  }

  /**
   * Get system information with native terminal profiles
   */
  static async getSystemInfo(): Promise<{
    platform: string;
    shell: string;
    workingDirectory: string;
    availableShells: string[];
    terminalProfiles: any;
  }> {
    try {
      return await invoke('get_system_info');
    } catch (error) {
      console.error('Failed to get system info:', error);
      return {
        platform: 'unknown',
        shell: 'bash',
        workingDirectory: '/',
        availableShells: ['bash'],
        terminalProfiles: {}
      };
    }
  }

  /**
   * Resize terminal
   */
  static async resizeTerminal(processId: string, cols: number, rows: number): Promise<void> {
    try {
      await invoke('resize_terminal', { processId, cols, rows });
    } catch (error) {
      console.error('Failed to resize terminal:', error);
      throw error;
    }
  }

  /**
   * Execute command in context (for cross-domain usage)
   */
  static async executeInContext(context: TerminalContext, command: string): Promise<TerminalCommand> {
    try {
      const result = await this.executeCommand(command, context.workingDirectory);
      
      return {
        id: crypto.randomUUID(),
        processId: 'context-execution',
        command,
        timestamp: new Date(),
        status: 'completed',
        output: result
      };
    } catch (error) {
      return {
        id: crypto.randomUUID(),
        processId: 'context-execution',
        command,
        timestamp: new Date(),
        status: 'failed',
        output: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private static setupOutputListener(processId: string): void {
    listen<TerminalOutput>('terminal-output', (event) => {
      const output = event.payload;
      if (output.process_id === processId) {
        // Call registered callback
        const callback = this.outputCallbacks.get(processId);
        if (callback) {
          callback(output);
        }

        // Run output parsers
        this.parseOutput(output).catch(console.error);
      }
    });
  }

  private static async checkCommandInterception(processId: string, input: string): Promise<boolean> {
    const process = await this.getProcess(processId);
    if (!process) return false;

    for (const interceptor of this.commandInterceptors) {
      if (interceptor.pattern.test(input)) {
        const intercepted = await interceptor.handler(input, process);
        if (intercepted) {
          return true;
        }
      }
    }
    return false;
  }

  private static async parseOutput(output: TerminalOutput): Promise<void> {
    for (const parser of this.outputParsers) {
      if (parser.pattern.test(output.content)) {
        // Get the process for the parser
        const process = await this.getProcess(output.process_id);
        if (process) {
          parser.handler(output.content, process);
        }
      }
    }
  }
}
