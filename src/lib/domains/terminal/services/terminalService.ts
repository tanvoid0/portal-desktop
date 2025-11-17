/**
 * Terminal Service - Domain-Specific Backend Integration
 * High-level API for terminal operations with full Tauri backend integration
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { TerminalProcess, TerminalOutput, TerminalCommand, TerminalContext, CreateProcessRequest } from '../types';
import { commandHistoryStore } from '../stores/commandHistoryStore';
import { patternCollector } from '@/lib/domains/learning';

export interface ExecuteCommandRequest {
  command: string;
  workingDirectory?: string;
  environment?: Record<string, string>;
}

export class TerminalService {
  private static outputCallbacks = new Map<string, (output: TerminalOutput) => void>();
  private static currentCommand: { command: string; startTime: number; output: string; exitCode?: number } | null = null;
  private static globalListenerSetup = false;
  private static globalListenerUnsubscribe: (() => void) | null = null;

  /**
   * Create a new terminal process with full control
   */
  static async createProcess(tabId: string, config: Partial<CreateProcessRequest> = {}): Promise<TerminalProcess> {
    // Create a proper environment for terminal applications
    const isWindows = navigator.userAgent.includes('Windows');
    const environment: Record<string, string> = {
      // Essential terminal capabilities - use a proper terminal type for TUI apps
      TERM: 'xterm-256color',
      COLORTERM: 'truecolor',
      
      // Shell environment - prefer zsh on Linux
      SHELL: isWindows ? 'powershell.exe' : '/usr/bin/zsh',
      
      // User environment - these will be overridden by backend with actual system values
      HOME: isWindows ? 'C:\\Users\\user' : '/home/tan',
      USER: 'tan',
      USERNAME: 'tan',
      
      // Path environment - use system PATH
      PATH: isWindows 
        ? 'C:\\Windows\\System32;C:\\Windows;C:\\Windows\\System32\\WindowsPowerShell\\v1.0'
        : '/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/home/tan/.local/bin',
      
      // Terminal size
      COLUMNS: '80',
      LINES: '24',
      
      // Locale
      LANG: 'en_US.UTF-8',
      LC_ALL: 'en_US.UTF-8',
      
      // Enable color output for TUI applications
      NO_COLOR: '0',
      FORCE_COLOR: '1',
      
      // Additional environment variables for better shell experience
      HISTSIZE: '10000',
      HISTFILESIZE: '10000',
      EDITOR: 'nano',
      PAGER: 'less',
    };

    const defaultConfig: CreateProcessRequest = {
      tab_id: tabId,
      shell: navigator.userAgent.includes('Windows') ? 'powershell.exe' : 'zsh',
      working_directory: navigator.userAgent.includes('Windows') ? 'C:\\' : '/home/tan',
      environment,
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
   * Send input directly to backend
   */
  static async sendInput(processId: string, input: string, tabId?: string): Promise<void> {
    try {
      // Track command line for history (but don't intercept)
      if (input === '\r\n' || input === '\n' || input === '\r') {
        const commandLine = this.lastCommandLine.trim();
        
        if (commandLine) {
          // Start tracking this command for history
          this.currentCommand = {
            command: commandLine,
            startTime: Date.now(),
            output: ''
          };
        }
        
        // Clear the command line after execution
        this.lastCommandLine = '';
      } else if (input === '\u007f' || input === '\b' || input === '\x7f' || input.charCodeAt(0) === 127 || input.charCodeAt(0) === 8) {
        // Backspace - remove last character
        this.lastCommandLine = this.lastCommandLine.slice(0, -1);
      } else if (input.length === 1 && input >= ' ' && input.charCodeAt(0) < 127) {
        // Regular printable character input - append to command line
        this.lastCommandLine += input;
      }

      // Send input directly to backend
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
    // Ensure global listener is set up
    await this.setupGlobalOutputListener();
    
    // Register the callback
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

  /**
   * Set up a global listener for terminal output events
   * This ensures we have a single listener that routes events to all registered callbacks
   */
  private static async setupGlobalOutputListener(): Promise<void> {
    if (this.globalListenerSetup) {
      return; // Already set up
    }

    try {
      const unsubscribe = await listen<TerminalOutput>('terminal-output', (event) => {
        const output = event.payload;
        const processId = output.process_id;

        // Capture output for current command
        if (this.currentCommand) {
          this.currentCommand.output += output.content;
        }

        // Handle exit events - get the actual exit code from the backend
        if (output.output_type === 'exit') {
          this.handleProcessExit(processId).catch(console.error);
        }

        // Call registered callback for this process
        const callback = this.outputCallbacks.get(processId);
        if (callback) {
          callback(output);
        }
      });

      this.globalListenerUnsubscribe = unsubscribe;
      this.globalListenerSetup = true;
    } catch (error) {
      console.error('Failed to set up global terminal output listener:', error);
    }
  }

  private static setupOutputListener(processId: string): void {
    // Ensure global listener is set up
    this.setupGlobalOutputListener().catch(console.error);
  }

  /**
   * Handle process exit and get the actual exit code
   */
  private static async handleProcessExit(processId: string): Promise<void> {
    try {
      // Get the process to extract tab_id
      const process = await this.getProcess(processId);
      const tabId = process?.tab_id;
      
      // Get the actual exit code from the backend
      const exitCode = await invoke<number | null>('get_process_exit_code', { processId });
      console.log(`Process ${processId} exited with code:`, exitCode);
      
      // Complete the current command with the real exit code
      await this.completeCurrentCommand(exitCode ?? undefined, tabId);
    } catch (error) {
      console.error('Failed to get process exit code:', error);
      // Fallback to output-based detection
      const process = await this.getProcess(processId);
      const tabId = process?.tab_id;
      await this.completeCurrentCommand(undefined, tabId);
    }
  }


  private static lastCommandLine: string = '';

  /**
   * Start tracking a command for history (used by quick commands)
   */
  static startCommandTracking(command: string, tabId?: string): void {
    console.log('Starting command tracking for:', command, 'tabId:', tabId);
    this.currentCommand = {
      command: command.trim(),
      startTime: Date.now(),
      output: ''
    };
    this.lastCommandLine = command.trim();
  }

  /**
   * Complete the current command and add it to history
   */
  static async completeCurrentCommand(exitCode?: number, tabId?: string): Promise<void> {
    // console.log('Completing current command:', this.currentCommand?.command, 'tabId:', tabId);
    if (this.currentCommand) {
      const duration = Date.now() - this.currentCommand.startTime;
      
      // Try to detect exit code from output if not provided
      const detectedExitCode = exitCode ?? this.detectExitCodeFromOutput(this.currentCommand.output);
      
      console.log('Adding to history - command:', this.currentCommand.command, 'output length:', this.currentCommand.output.length, 'tabId:', tabId);
      
      if (tabId) {
        // Use persistence-enabled method
        commandHistoryStore.addEntryWithPersistence(tabId, {
          command: this.currentCommand.command,
          output: this.currentCommand.output,
          exitCode: detectedExitCode,
          duration,
          intercepted: false
        });
      }

      // Learn from command pattern
      try {
        const success = detectedExitCode === 0 || detectedExitCode === undefined;
        // Extract context from working directory if available
        const context = undefined; // Could be extracted from working directory
        await patternCollector.collectCommandPattern(
          this.currentCommand.command,
          success,
          context
        );
      } catch (error) {
        // Don't fail command completion if learning fails
        console.warn('Failed to collect command pattern', error);
      }
      
      this.currentCommand = null;
    }
  }

  /**
   * Detect command success/failure from output content
   */
  private static detectExitCodeFromOutput(output: string): number | undefined {
    if (!output) return undefined;
    
    // Common error patterns
    const errorPatterns = [
      /'[^']+' is not recognized as an internal or external command/i,
      /command not found/i,
      /permission denied/i,
      /access denied/i,
      /file not found/i,
      /no such file or directory/i,
      /error:/i,
      /failed/i,
      /cannot/i,
      /unable to/i
    ];
    
    // Check if output contains error patterns
    for (const pattern of errorPatterns) {
      if (pattern.test(output)) {
        return 1; // Non-zero exit code for errors
      }
    }
    
    // If no error patterns found, assume success
    return 0;
  }

}
