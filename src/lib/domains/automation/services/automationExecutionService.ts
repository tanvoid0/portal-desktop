/**
 * Automation Execution Service
 *
 * Simple API for any directory or project:
 *
 *   import { automation } from '$lib/domains/automation';
 *
 *   // Run in any folder
 *   await automation.run({ cwd: 'D:/my-app', blocks: ['install-npm', 'test-npm'] });
 *
 *   // Bound to a directory
 *   const runner = automation.in('D:/my-app');
 *   await runner.run(['install-npm']);
 *   const cmd = await runner.command('install-npm', { packageManager: 'pnpm' });
 *
 *   // From a project
 *   await automation.forProject(project).run(presets.ci());
 */

import { logger } from "$lib/domains/shared";
import { blockLibraryService } from "$lib/domains/projects/pipelines/services/blockLibraryService";
import { executionService } from "$lib/domains/projects/pipelines/services/executionService";
import { pipelineService } from "$lib/domains/projects/pipelines/services/pipelineService";
import {
  scriptExecutionService,
  type ScriptExecutionInfo,
} from "$lib/domains/scripts/services/scriptExecutionService";
import type {
  ExecutePipelineRequest,
  Pipeline,
  PipelineExecution,
} from "$lib/domains/projects/pipelines/types";
import type {
  AutomationStepInput,
  ResolveContext,
  ResolvePlanResult,
  ResolvedExecutionStep,
} from "../utils/blockResolver";
import {
  buildExecutionPlan,
  materializePipelineSteps,
  pipelineVariablesToRecord,
} from "../utils/blockResolver";
import {
  createAutomationContext,
  type AutomationContextOptions,
  type AutomationProjectRef,
} from "../utils/automationContext";
import {
  normalizeStepRefs,
  presets,
  type AutomationStepRef,
} from "../utils/stepRefs";

const log = logger.createScoped("AutomationExecutionService");

// ─── Public types ───────────────────────────────────────────────────────────

export interface RunAutomationOptions {
  /** Any directory to run commands in */
  cwd: string;
  /** Block IDs or step configs — e.g. `['install-npm', { blockId: 'test-npm' }]` */
  blocks: AutomationStepRef[];
  variables?: Record<string, string | number | boolean>;
  secrets?: Record<string, string>;
  stopOnError?: boolean;
  /** Optional: legacy — ignored for execution path (no throwaway pipelines) */
  projectId?: string;
  project?: AutomationProjectRef;
}

export interface ResolveAutomationOptions {
  cwd: string;
  blocks: AutomationStepRef[];
  variables?: Record<string, string | number | boolean>;
  secrets?: Record<string, string>;
  project?: AutomationProjectRef;
}

export interface AutomationStepResult {
  name: string;
  command: string;
  blockId: string;
  status: "success" | "failed" | "skipped";
  executionId?: string;
  exitCode?: number | null;
  output?: string;
  error?: string;
}

export interface AutomationRunResult {
  success: boolean;
  cwd: string;
  steps: AutomationStepResult[];
}

export interface AutomationPlanPreview extends ResolvePlanResult {
  ready: boolean;
}

export interface AutomationRunner {
  readonly cwd: string;
  resolve(
    blocks: AutomationStepRef[],
    variables?: Record<string, string | number | boolean>,
  ): Promise<ResolvedExecutionStep[]>;
  command(
    blockId: string,
    parameters?: Record<string, string | number | boolean>,
  ): Promise<string>;
  run(
    blocks: AutomationStepRef[],
    options?: {
      variables?: Record<string, string | number | boolean>;
      stopOnError?: boolean;
    },
  ): Promise<AutomationRunResult>;
}

// ─── Service ────────────────────────────────────────────────────────────────

class AutomationExecutionService {
  private static instance: AutomationExecutionService;

  static getInstance(): AutomationExecutionService {
    if (!AutomationExecutionService.instance) {
      AutomationExecutionService.instance = new AutomationExecutionService();
    }
    return AutomationExecutionService.instance;
  }

  private buildContext(options: AutomationContextOptions): ResolveContext {
    return createAutomationContext(options);
  }

  async loadBlocks() {
    return blockLibraryService.getBlocks();
  }

  async preview(
    options: ResolveAutomationOptions,
  ): Promise<AutomationPlanPreview> {
    const blocks = await this.loadBlocks();
    const context = this.buildContext({
      cwd: options.cwd,
      variables: options.variables,
      secrets: options.secrets,
      project: options.project,
    });
    const result = buildExecutionPlan(
      normalizeStepRefs(options.blocks),
      blocks,
      context,
    );
    return {
      ...result,
      ready: result.errors.length === 0 && result.steps.length > 0,
    };
  }

  async resolve(options: ResolveAutomationOptions): Promise<ResolvedExecutionStep[]> {
    const preview = await this.preview(options);
    if (!preview.ready) {
      throw new Error(preview.errors.join("; "));
    }
    if (preview.warnings.length > 0) {
      log.warn("Resolved with warnings", { warnings: preview.warnings });
    }
    return preview.steps;
  }

  /** Run blocks in any directory — no project or pipeline required.
   * Prefer `actions.run` for the unified Actions API. */
  async run(options: RunAutomationOptions): Promise<AutomationRunResult> {
    const stopOnError = options.stopOnError ?? true;

    // Always use script execution — never create throwaway DB pipelines
    const plan = await this.resolve({
      cwd: options.cwd,
      blocks: options.blocks,
      variables: options.variables,
      secrets: options.secrets,
      project: options.project,
    });

    const results: AutomationStepResult[] = [];
    let success = true;

    for (const step of plan) {
      if (!success && stopOnError) {
        results.push({
          name: step.name,
          command: step.command,
          blockId: step.blockId,
          status: "skipped",
        });
        continue;
      }

      try {
        const executionId = await scriptExecutionService.executeScript({
          blockId: step.blockId,
          command: step.command,
          parameters: step.parameters,
          workingDirectory: step.workingDirectory ?? options.cwd,
        });

        const finished = await this.waitForExecution(executionId, step.longRunning);
        const stepSuccess = finished.status === "success";
        const failureError = stepSuccess
          ? undefined
          : finished.error?.trim() ||
            (finished.exitCode != null
              ? `Exit code ${finished.exitCode}`
              : undefined) ||
            finished.output
              ?.split(/\r?\n/)
              .map((l) => l.trim())
              .filter(Boolean)
              .slice(-6)
              .join("\n") ||
            `Execution ${finished.status}`;

        results.push({
          name: step.name,
          command: step.command,
          blockId: step.blockId,
          status: stepSuccess ? "success" : "failed",
          executionId,
          exitCode: finished.exitCode,
          output: finished.output,
          error: failureError,
        });

        if (!stepSuccess) {
          success = false;
          if (stopOnError) break;
        }
      } catch (error) {
        success = false;
        results.push({
          name: step.name,
          command: step.command,
          blockId: step.blockId,
          status: "failed",
          error: error instanceof Error ? error.message : String(error),
        });
        if (stopOnError) break;
      }
    }

    return { success, cwd: options.cwd, steps: results };
  }

  /** Directory-bound runner for integrations */
  in(cwd: string, project?: RunAutomationOptions["project"]): AutomationRunner {
    const service = this;
    return {
      cwd,
      resolve: (blocks, variables) =>
        service.resolve({ cwd, blocks, variables, project }),
      command: (blockId, parameters) =>
        service.resolveBlockCommand(
          blockId,
          parameters ?? {},
          createAutomationContext({ cwd, project, variables: parameters }),
        ),
      run: (blocks, opts) =>
        service.run({
          cwd,
          blocks,
          variables: opts?.variables,
          stopOnError: opts?.stopOnError,
          project,
        }),
    };
  }

  forProject(
    project: AutomationProjectRef,
  ): AutomationRunner & { runAsPipeline: (blocks: AutomationStepRef[]) => Promise<AutomationRunResult> } {
    const runner = this.in(project.path, project);
    return {
      ...runner,
      /** @deprecated Use actions.forProject(project).run(...) — no longer creates DB pipelines */
      runAsPipeline: (blocks) =>
        this.run({
          cwd: project.path,
          blocks,
          project,
        }),
    };
  }

  async resolveBlockCommand(
    blockId: string,
    parameters: Record<string, string | number | boolean> = {},
    context: ResolveContext = {},
  ): Promise<string> {
    const blocks = await this.loadBlocks();
    const block = blocks.find((b) => b.id === blockId);
    if (!block) {
      throw new Error(`Block not found: ${blockId}`);
    }

    const plan = buildExecutionPlan([{ blockId, parameters }], blocks, context);
    if (plan.errors.length > 0) {
      throw new Error(plan.errors.join("; "));
    }
    return plan.steps[0]?.command ?? "";
  }

  async ensurePipelineReady(
    pipeline: Pipeline,
    runtimeVariables: Record<string, string> = {},
  ): Promise<Pipeline> {
    const blocks = await this.loadBlocks();
    const context: ResolveContext = {
      variables: {
        ...pipelineVariablesToRecord(pipeline.variables),
        ...runtimeVariables,
      },
    };

    const needsResolution = pipeline.steps.some(
      (s) => !(s.config as Record<string, unknown>)?.command,
    );
    if (!needsResolution) return pipeline;

    const { steps, errors } = materializePipelineSteps(
      pipeline.steps,
      blocks,
      context,
    );
    if (errors.length > 0) {
      throw new Error(`Cannot execute pipeline: ${errors.join("; ")}`);
    }

    return pipelineService.updatePipeline(pipeline.id, { steps });
  }

  async executePipeline(
    request: ExecutePipelineRequest,
  ): Promise<PipelineExecution> {
    const pipeline = await pipelineService.getPipeline(request.pipelineId);
    if (!pipeline) throw new Error("Pipeline not found");
    await this.ensurePipelineReady(pipeline, request.variables);
    return executionService.executePipeline(request);
  }

  // ─── Private ──────────────────────────────────────────────────────────────

  private async waitForExecution(
    executionId: string,
    longRunning = false,
  ): Promise<ScriptExecutionInfo> {
    const pollMs = 500;
    const maxWait = longRunning ? undefined : 30 * 60 * 1000;
    const start = Date.now();

    for (;;) {
      const exec = await scriptExecutionService.getExecution(executionId);
      if (!exec) throw new Error(`Execution not found: ${executionId}`);

      if (["success", "failed", "cancelled"].includes(exec.status)) {
        return exec;
      }

      if (maxWait !== undefined && Date.now() - start > maxWait) {
        throw new Error(`Execution timed out: ${executionId}`);
      }

      await new Promise((r) => setTimeout(r, pollMs));
    }
  }
}

export const automationExecutionService =
  AutomationExecutionService.getInstance();

/** Primary public API */
export const automation = {
  /** Run blocks in any directory */
  run: (options: RunAutomationOptions) =>
    automationExecutionService.run(options),

  /** Resolve blocks to commands without running */
  resolve: (options: ResolveAutomationOptions) =>
    automationExecutionService.resolve(options),

  /** Preview plan with errors/warnings */
  preview: (options: ResolveAutomationOptions) =>
    automationExecutionService.preview(options),

  /** Bind to a working directory */
  in: (cwd: string, project?: RunAutomationOptions["project"]) =>
    automationExecutionService.in(cwd, project),

  /** Bind to a registered project */
  forProject: (project: AutomationProjectRef) =>
    automationExecutionService.forProject(project),

  /** Common block sequences */
  presets,

  /** List available blocks */
  loadBlocks: () => automationExecutionService.loadBlocks(),
};

// Legacy aliases
export type RunBlockSequenceOptions = RunAutomationOptions;
export type { AutomationStepRef } from "../utils/stepRefs";
