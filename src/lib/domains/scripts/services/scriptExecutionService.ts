/**
 * Script Execution Service - Manages script execution lifecycle
 *
 * Provides persistent script execution tracking that survives app restarts.
 * Executions are stored in the database with their status, output, and process info.
 */

import { invokeClient } from "$lib/utils/invokeClient";
import { logger } from "$lib/domains/shared";

const log = logger.createScoped("ScriptExecutionService");

export interface ScriptExecutionInfo {
  id: string;
  blockId: string | null;
  command: string;
  parameters: Record<string, string>;
  workingDirectory: string | null;
  status: "pending" | "running" | "success" | "failed" | "cancelled";
  exitCode: number | null;
  pid: number | null;
  output: string;
  error: string | null;
  startedAt: string;
  finishedAt: string | null;
  triggeredBy: string;
}

export interface ExecuteScriptParams {
  blockId?: string;
  command: string;
  parameters?: Record<string, string>;
  workingDirectory?: string;
}

class ScriptExecutionService {
  private static instance: ScriptExecutionService;
  private pollingIntervals: Map<string, number> = new Map();

  static getInstance(): ScriptExecutionService {
    if (!ScriptExecutionService.instance) {
      ScriptExecutionService.instance = new ScriptExecutionService();
    }
    return ScriptExecutionService.instance;
  }

  /**
   * Execute a script and return the execution ID
   */
  async executeScript(params: ExecuteScriptParams): Promise<string> {
    try {
      log.info("Executing script", {
        command: params.command,
        blockId: params.blockId,
      });

      // Note: Tauri 2.0 uses camelCase for parameter names in JS/TS
      const executionId = await invokeClient.post<string>("execute_script", {
        params: {
          blockId: params.blockId,
          command: params.command,
          parameters: params.parameters || {},
          workingDirectory: params.workingDirectory,
        },
      });

      log.info("Script execution started", { executionId });
      return executionId;
    } catch (error) {
      log.error("Failed to execute script", { error });
      throw error;
    }
  }

  /**
   * Get execution details by ID
   */
  async getExecution(executionId: string): Promise<ScriptExecutionInfo | null> {
    try {
      const execution = await invokeClient.post<ScriptExecutionInfo | null>(
        "get_script_execution",
        {
          executionId,
        },
      );

      return execution ? this.mapExecutionResponse(execution) : null;
    } catch (error) {
      log.error("Failed to get execution", { executionId, error });
      throw error;
    }
  }

  /**
   * Get live output for a running execution
   * This returns buffered output that hasn't been persisted yet
   */
  async getLiveOutput(executionId: string): Promise<string[]> {
    try {
      const output = await invokeClient.post<string[]>(
        "get_script_execution_live_output",
        {
          executionId,
        },
      );
      return output;
    } catch (error) {
      log.error("Failed to get live output", { executionId, error });
      return [];
    }
  }

  /**
   * Cancel a running execution
   */
  async cancelExecution(executionId: string): Promise<void> {
    try {
      log.info("Cancelling execution", { executionId });
      await invokeClient.post("cancel_script_execution", {
        executionId,
      });
      log.info("Execution cancelled", { executionId });
    } catch (error) {
      log.error("Failed to cancel execution", { executionId, error });
      throw error;
    }
  }

  /**
   * Get executions for a specific block/script
   */
  async getExecutionsByBlock(
    blockId: string,
    limit?: number,
  ): Promise<ScriptExecutionInfo[]> {
    try {
      const executions = await invokeClient.post<ScriptExecutionInfo[]>(
        "get_script_executions_by_block",
        {
          blockId,
          limit,
        },
      );
      return executions.map((e) => this.mapExecutionResponse(e));
    } catch (error) {
      log.error("Failed to get executions by block", { blockId, error });
      return [];
    }
  }

  /**
   * Get all currently running executions
   */
  async getRunningExecutions(): Promise<ScriptExecutionInfo[]> {
    try {
      const executions = await invokeClient.post<ScriptExecutionInfo[]>(
        "get_running_script_executions",
      );
      return executions.map((e) => this.mapExecutionResponse(e));
    } catch (error) {
      log.error("Failed to get running executions", { error });
      return [];
    }
  }

  /**
   * Get recent executions
   */
  async getRecentExecutions(
    limit: number = 20,
  ): Promise<ScriptExecutionInfo[]> {
    try {
      const executions = await invokeClient.post<ScriptExecutionInfo[]>(
        "get_recent_script_executions",
        {
          limit,
        },
      );
      return executions.map((e) => this.mapExecutionResponse(e));
    } catch (error) {
      log.error("Failed to get recent executions", { error });
      return [];
    }
  }

  /**
   * Sync running executions on app startup
   * This checks if processes are still running and updates their status
   */
  async syncExecutions(): Promise<void> {
    try {
      log.info("Syncing script executions");
      await invokeClient.post("sync_script_executions");
      log.info("Script executions synced");
    } catch (error) {
      log.error("Failed to sync executions", { error });
    }
  }

  /**
   * Delete an execution record
   */
  async deleteExecution(executionId: string): Promise<void> {
    try {
      log.info("Deleting execution", { executionId });
      await invokeClient.post("delete_script_execution", {
        executionId,
      });
      log.info("Execution deleted", { executionId });
    } catch (error) {
      log.error("Failed to delete execution", { executionId, error });
      throw error;
    }
  }

  /**
   * Subscribe to execution updates with polling
   * Returns an unsubscribe function
   */
  subscribeToExecution(
    executionId: string,
    callback: (execution: ScriptExecutionInfo) => void,
    intervalMs: number = 1000,
  ): () => void {
    // Clear any existing interval for this execution
    this.unsubscribeFromExecution(executionId);

    const pollInterval = setInterval(async () => {
      try {
        const execution = await this.getExecution(executionId);
        if (execution) {
          callback(execution);

          // Auto-unsubscribe when execution is complete
          if (
            execution.status === "success" ||
            execution.status === "failed" ||
            execution.status === "cancelled"
          ) {
            this.unsubscribeFromExecution(executionId);
          }
        }
      } catch (error) {
        log.error("Error polling execution", { executionId, error });
      }
    }, intervalMs);

    this.pollingIntervals.set(executionId, pollInterval as unknown as number);

    // Return unsubscribe function
    return () => this.unsubscribeFromExecution(executionId);
  }

  /**
   * Unsubscribe from execution updates
   */
  unsubscribeFromExecution(executionId: string): void {
    const interval = this.pollingIntervals.get(executionId);
    if (interval) {
      clearInterval(interval);
      this.pollingIntervals.delete(executionId);
    }
  }

  /**
   * Clear all subscriptions
   */
  clearAllSubscriptions(): void {
    this.pollingIntervals.forEach((interval) => clearInterval(interval));
    this.pollingIntervals.clear();
  }

  /**
   * Map backend response to frontend interface (handle snake_case to camelCase)
   */
  private mapExecutionResponse(execution: any): ScriptExecutionInfo {
    return {
      id: execution.id,
      blockId: execution.block_id ?? execution.blockId ?? null,
      command: execution.command,
      parameters: execution.parameters || {},
      workingDirectory:
        execution.working_directory ?? execution.workingDirectory ?? null,
      status: execution.status,
      exitCode: execution.exit_code ?? execution.exitCode ?? null,
      pid: execution.pid ?? null,
      output: execution.output || "",
      error: execution.error ?? null,
      startedAt: execution.started_at ?? execution.startedAt,
      finishedAt: execution.finished_at ?? execution.finishedAt ?? null,
      triggeredBy: execution.triggered_by ?? execution.triggeredBy ?? "user",
    };
  }
}

export const scriptExecutionService = ScriptExecutionService.getInstance();
