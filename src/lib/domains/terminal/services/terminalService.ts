/**
 * Terminal Service - Domain-Specific Backend Integration
 * High-level API for terminal operations with full Tauri backend integration
 */

import { invokeClient } from "$lib/utils/invokeClient";
import { isTauriEnvironment } from "$lib/utils/tauri";
import type {
  TerminalProcess,
  TerminalOutput,
  TerminalCommand,
  TerminalContext,
  CreateProcessRequest,
} from "../types";
import { commandHistoryStore } from "../stores/commandHistoryStore";
import { patternCollector } from "$lib/domains/learning";
import {
  subscribeTerminalOutput,
  ensureTerminalOutputListener,
} from "./terminalOutputBus";

export interface ExecuteCommandRequest {
  command: string;
  workingDirectory?: string;
  environment?: Record<string, string>;
}

export class TerminalService {
  private static currentCommand: {
    command: string;
    startTime: number;
    output: string;
    exitCode?: number;
  } | null = null;

  /**
   * Create a new terminal process with full control
   */
  static async createProcess(
    tabId: string,
    config: Partial<CreateProcessRequest> = {},
  ): Promise<TerminalProcess> {
    // The Rust backend fills in the real environment (HOME/USER/PATH/TERM/…)
    // from the actual process. We only forward caller-supplied overrides — no
    // fake hardcoded values that would clobber the user's real environment.
    const defaultConfig: CreateProcessRequest = {
      tab_id: tabId,
      shell: navigator.userAgent.includes("Windows") ? "powershell.exe" : "zsh",
      // Empty → backend falls back to the app's current working directory.
      working_directory: "",
      environment: {},
      cols: 80,
      rows: 24,
      ...config,
    };

    try {
      const process = await invokeClient.request<TerminalProcess>(
        "create_terminal_process",
        {
          data: {
            request: defaultConfig,
          },
        },
      );

      // Set up output listener for this process
      this.setupOutputListener(process.id);

      return process;
    } catch (error) {
      console.error("Failed to create terminal process:", error);
      throw error;
    }
  }

  /**
   * Send input directly to backend
   */
  static async sendInput(
    processId: string,
    input: string,
    tabId?: string,
  ): Promise<void> {
    try {
      // Track command line for history (but don't intercept)
      if (input === "\r\n" || input === "\n" || input === "\r") {
        const commandLine = this.lastCommandLine.trim();

        if (commandLine) {
          // Start tracking this command for history
          this.currentCommand = {
            command: commandLine,
            startTime: Date.now(),
            output: "",
          };
        }

        // Clear the command line after execution
        this.lastCommandLine = "";
      } else if (
        input === "\u007f" ||
        input === "\b" ||
        input === "\x7f" ||
        input.charCodeAt(0) === 127 ||
        input.charCodeAt(0) === 8
      ) {
        // Backspace - remove last character
        this.lastCommandLine = this.lastCommandLine.slice(0, -1);
      } else if (
        input.length === 1 &&
        input >= " " &&
        input.charCodeAt(0) < 127
      ) {
        // Regular printable character input - append to command line
        this.lastCommandLine += input;
      }

      // Send input directly to backend
      await invokeClient.request("send_terminal_input", {
        data: {
          processId,
          input,
        },
      });
    } catch (error) {
      console.error("Failed to send input:", error);
      throw error;
    }
  }

  /**
   * Kill process with full cleanup
   */
  static async killProcess(processId: string): Promise<void> {
    try {
      await invokeClient.request("kill_terminal_process", {
        data: { processId },
      });
    } catch (error) {
      console.error("Failed to kill process:", error);
      throw error;
    }
  }

  /**
   * Get process information
   */
  static async getProcess(processId: string): Promise<TerminalProcess | null> {
    try {
      return await invokeClient.request<TerminalProcess | null>(
        "get_terminal_process",
        { data: { processId } },
      );
    } catch (error) {
      console.error("Failed to get process:", error);
      return null;
    }
  }

  /**
   * Get all processes
   */
  static async getAllProcesses(): Promise<TerminalProcess[]> {
    try {
      return await invokeClient.request<TerminalProcess[]>(
        "get_terminal_processes",
      );
    } catch (error) {
      console.error("Failed to get processes:", error);
      return [];
    }
  }

  /**
   * Subscribe to output with full control
   */
  static async subscribeToOutput(
    processId: string,
    callback: (output: TerminalOutput) => void,
  ): Promise<() => void> {
    await ensureTerminalOutputListener();

    return subscribeTerminalOutput(processId, (output) => {
      if (this.currentCommand) {
        this.currentCommand.output += output.content;
      }

      if (output.output_type === "exit") {
        this.handleProcessExit(processId).catch(console.error);
      }

      callback(output);
    });
  }

  /**
   * Execute command with interception
   */
  static async executeCommand(
    command: string,
    workingDirectory?: string,
  ): Promise<string> {
    try {
      const request: ExecuteCommandRequest = {
        command,
        workingDirectory,
        environment: {},
      };

      return await invokeClient.request<string>("execute_command", {
        data: { request },
      });
    } catch (error) {
      console.error("Failed to execute command:", error);
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
      return await invokeClient.request("get_system_info");
    } catch (error) {
      console.error("Failed to get system info:", error);
      return {
        platform: "unknown",
        shell: "bash",
        workingDirectory: "/",
        availableShells: ["bash"],
        terminalProfiles: {},
      };
    }
  }

  /**
   * Resize terminal
   */
  static async resizeTerminal(
    processId: string,
    cols: number,
    rows: number,
  ): Promise<void> {
    try {
      await invokeClient.request("resize_terminal", {
        data: { processId, cols, rows },
      });
    } catch (error) {
      console.error("Failed to resize terminal:", error);
      throw error;
    }
  }

  /**
   * Execute command in context (for cross-domain usage)
   */
  static async executeInContext(
    context: TerminalContext,
    command: string,
    options?: { captureBlock?: boolean },
  ): Promise<TerminalCommand> {
    try {
      const result = await this.executeCommand(
        command,
        context.workingDirectory,
      );

      const cmd: TerminalCommand = {
        id: crypto.randomUUID(),
        processId: "context-execution",
        command,
        timestamp: new Date(),
        status: "completed",
        output: result,
      };

      if (options?.captureBlock !== false) {
        const { commandBlockStore } = await import(
          "../stores/commandBlockStore"
        );
        commandBlockStore.captureReadonlyResult(context.tabId, {
          ...cmd,
          exitCode: 0,
        });
      }

      return cmd;
    } catch (error) {
      const cmd: TerminalCommand = {
        id: crypto.randomUUID(),
        processId: "context-execution",
        command,
        timestamp: new Date(),
        status: "failed",
        output: error instanceof Error ? error.message : "Unknown error",
      };

      if (options?.captureBlock !== false) {
        const { commandBlockStore } = await import(
          "../stores/commandBlockStore"
        );
        commandBlockStore.captureReadonlyResult(context.tabId, {
          ...cmd,
          exitCode: 1,
        });
      }

      return cmd;
    }
  }

  private static setupOutputListener(_processId: string): void {
    ensureTerminalOutputListener().catch(console.error);
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
      const exitCode = await invokeClient.request<number | null>(
        "get_process_exit_code",
        { data: { processId } },
      );
      console.log(`Process ${processId} exited with code:`, exitCode);

      // Complete the current command with the real exit code
      await this.completeCurrentCommand(exitCode ?? undefined, tabId);
    } catch (error) {
      console.error("Failed to get process exit code:", error);
      // Fallback to output-based detection
      const process = await this.getProcess(processId);
      const tabId = process?.tab_id;
      await this.completeCurrentCommand(undefined, tabId);
    }
  }

  private static lastCommandLine: string = "";

  /**
   * Start tracking a command for history (used by quick commands)
   */
  static startCommandTracking(command: string, tabId?: string): void {
    console.log("Starting command tracking for:", command, "tabId:", tabId);
    this.currentCommand = {
      command: command.trim(),
      startTime: Date.now(),
      output: "",
    };
    this.lastCommandLine = command.trim();
  }

  /**
   * Complete the current command and add it to history
   */
  static async completeCurrentCommand(
    exitCode?: number,
    tabId?: string,
  ): Promise<void> {
    // console.log('Completing current command:', this.currentCommand?.command, 'tabId:', tabId);
    if (this.currentCommand) {
      const duration = Date.now() - this.currentCommand.startTime;

      // Exit code comes from the backend (get_process_exit_code / OSC 133).
      // The old "output contains the word error → exit 1" heuristic was removed:
      // it produced false failures for any command that merely printed "error".
      const detectedExitCode = exitCode;

      if (tabId) {
        // Use persistence-enabled method
        commandHistoryStore.addEntryWithPersistence(tabId, {
          command: this.currentCommand.command,
          output: this.currentCommand.output,
          exitCode: detectedExitCode,
          duration,
          intercepted: false,
        });
      }

      // Learn from command pattern
      try {
        const success =
          detectedExitCode === 0 || detectedExitCode === undefined;
        // Extract context from working directory if available
        const context = undefined; // Could be extracted from working directory
        await patternCollector.collectCommandPattern(
          this.currentCommand.command,
          success,
          context,
        );
      } catch (error) {
        // Don't fail command completion if learning fails
        console.warn("Failed to collect command pattern", error);
      }

      this.currentCommand = null;
    }
  }

  /**
   * Fetch the real exit code recorded by the backend for a process.
   */
  static async getProcessExitCode(processId: string): Promise<number | null> {
    try {
      return await invokeClient.request<number | null>(
        "get_process_exit_code",
        { data: { processId } },
      );
    } catch {
      return null;
    }
  }
}
