/**
 * Unified Actions run API — single / list / workflow across local, GitHub, n8n.
 */

import { logger } from "$lib/domains/shared";
import type { Project } from "$lib/domains/projects/types";
import {
  scriptExecutionService,
  type ScriptExecutionInfo,
} from "$lib/domains/scripts/services/scriptExecutionService";
import {
  resolveDirectoryCatalog,
  resolveProjectCatalog,
  type ActionCatalog,
  type ResolveCatalogOptions,
} from "./resolver";
import { profileFromProject, profileVariables } from "./profile";
import { createAutomationContext } from "$lib/domains/automation/utils/automationContext";
import { listGitHubActions } from "./adapters/githubAdapter";
import { listN8nActions, runN8nAction } from "./adapters/n8nAdapter";
import { runGitHubWorkflow } from "./adapters/githubAdapter";
import type {
  ActionPlanPreview,
  ActionRunOptions,
  ActionRunResult,
  ActionStepResult,
  UnifiedAction,
  UnifiedWorkflow,
} from "./types";

const log = logger.createScoped("ActionsService");

function summarizeExecutionFailure(exec: ScriptExecutionInfo): string {
  if (exec.error?.trim()) return exec.error.trim();

  const tail = (exec.output || "")
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)
    .slice(-6)
    .join("\n");

  if (tail) {
    return exec.exitCode != null
      ? `Exit code ${exec.exitCode}\n${tail}`
      : tail;
  }

  if (exec.status === "cancelled") return "Execution was cancelled";
  if (exec.exitCode != null) return `Exit code ${exec.exitCode}`;
  return `Execution ${exec.status}`;
}

export interface ProjectActionsHandle {
  list: () => Promise<UnifiedAction[]>;
  workflows: () => Promise<UnifiedWorkflow[]>;
  catalog: () => Promise<ActionCatalog>;
  run: (
    target: string | string[],
    options?: ActionRunOptions,
  ) => Promise<ActionRunResult>;
  preview: (
    target: string | string[],
    options?: ActionRunOptions,
  ) => Promise<ActionPlanPreview>;
}

function topoSortSteps(
  steps: UnifiedWorkflow["steps"],
  only?: string[],
): string[] {
  const filtered = only?.length
    ? steps.filter((s) => only.includes(s.action))
    : steps;
  const ids = new Set(filtered.map((s) => s.action));
  const needs = new Map(
    filtered.map((s) => [
      s.action,
      (s.needs ?? []).filter((n) => ids.has(n)),
    ]),
  );
  const result: string[] = [];
  const visited = new Set<string>();
  const visiting = new Set<string>();

  function visit(id: string) {
    if (visited.has(id)) return;
    if (visiting.has(id)) {
      throw new Error(`Circular dependency in workflow involving "${id}"`);
    }
    visiting.add(id);
    for (const dep of needs.get(id) ?? []) visit(dep);
    visiting.delete(id);
    visited.add(id);
    result.push(id);
  }

  for (const id of ids) visit(id);
  return result;
}

class ActionsService {
  private static instance: ActionsService;

  static getInstance(): ActionsService {
    if (!ActionsService.instance) {
      ActionsService.instance = new ActionsService();
    }
    return ActionsService.instance;
  }

  async catalogForProject(
    project: Project,
    options?: ResolveCatalogOptions,
  ): Promise<ActionCatalog> {
    const local = await resolveProjectCatalog(project, options);
    const [github, n8n] = await Promise.all([
      listGitHubActions(project).catch((err) => {
        log.warn("GitHub actions catalog failed", { err });
        return {
          actions: [] as UnifiedAction[],
          workflows: [] as UnifiedWorkflow[],
          warnings: [] as string[],
        };
      }),
      listN8nActions(project).catch((err) => {
        log.warn("n8n actions catalog failed", { err });
        return {
          actions: [] as UnifiedAction[],
          workflows: [] as UnifiedWorkflow[],
          warnings: [] as string[],
        };
      }),
    ]);

    return {
      ...local,
      actions: [...local.actions, ...github.actions, ...n8n.actions],
      workflows: [...local.workflows, ...github.workflows, ...n8n.workflows],
      warnings: [
        ...local.warnings,
        ...github.warnings,
        ...n8n.warnings,
      ],
    };
  }

  async catalogForDirectory(
    cwd: string,
    options?: ResolveCatalogOptions & { packageManager?: string },
  ): Promise<ActionCatalog> {
    return resolveDirectoryCatalog(cwd, options);
  }

  forProject(project: Project): ProjectActionsHandle {
    const service = this;
    return {
      list: async () => (await service.catalogForProject(project)).actions,
      workflows: async () =>
        (await service.catalogForProject(project)).workflows,
      catalog: () => service.catalogForProject(project),
      run: (target, options) =>
        service.run(target, { ...options, project }),
      preview: (target, options) =>
        service.preview(target, { ...options, project }),
    };
  }

  async preview(
    target: string | string[],
    options: ActionRunOptions & { project?: Project; cwd?: string } = {},
  ): Promise<ActionPlanPreview> {
    const plan = await this.resolveLocalPlan(target, options);
    return {
      ready: plan.errors.length === 0 && plan.steps.length > 0,
      steps: plan.steps,
      errors: plan.errors,
      warnings: plan.warnings,
    };
  }

  async run(
    target: string | string[],
    options: ActionRunOptions & { project?: Project; cwd?: string } = {},
  ): Promise<ActionRunResult> {
    const cwd =
      options.cwd ??
      options.project?.path ??
      (typeof target === "string" ? undefined : undefined);

    if (!cwd && !options.project) {
      throw new Error("cwd or project required to run actions");
    }

    const workingDir = cwd ?? options.project!.path;
    const catalog = options.project
      ? await this.catalogForProject(options.project)
      : await this.catalogForDirectory(workingDir);

    // Workflow or single action dispatch by runner
    if (typeof target === "string") {
      const workflow = catalog.workflows.find((w) => w.id === target);
      if (workflow) {
        return this.runWorkflow(workflow, catalog, {
          ...options,
          cwd: workingDir,
        });
      }
      const action = catalog.actions.find((a) => a.id === target);
      if (!action) {
        throw new Error(`Unknown action or workflow: ${target}`);
      }
      return this.runActionList([action], {
        ...options,
        cwd: workingDir,
        target,
      });
    }

    const actions = target.map((id) => {
      const action = catalog.actions.find((a) => a.id === id);
      if (!action) throw new Error(`Unknown action: ${id}`);
      return action;
    });
    return this.runActionList(actions, {
      ...options,
      cwd: workingDir,
      target,
    });
  }

  // ─── Internals ────────────────────────────────────────────────────────────

  private async resolveLocalPlan(
    target: string | string[],
    options: ActionRunOptions & { project?: Project; cwd?: string },
  ) {
    const workingDir = options.cwd ?? options.project?.path;
    if (!workingDir) {
      return {
        steps: [],
        errors: ["cwd or project required"],
        warnings: [] as string[],
      };
    }

    const catalog = options.project
      ? await this.catalogForProject(options.project)
      : await this.catalogForDirectory(workingDir);

    let actionIds: string[];
    if (typeof target === "string") {
      const workflow = catalog.workflows.find((w) => w.id === target);
      if (workflow) {
        if (workflow.runner !== "local") {
          return {
            steps: [],
            errors: [
              `Preview of remote workflow "${target}" (${workflow.runner}) is not available locally`,
            ],
            warnings: catalog.warnings,
          };
        }
        try {
          actionIds = topoSortSteps(workflow.steps, options.only);
        } catch (e) {
          return {
            steps: [],
            errors: [e instanceof Error ? e.message : String(e)],
            warnings: catalog.warnings,
          };
        }
      } else {
        actionIds = [target];
      }
    } else {
      actionIds = target;
    }

    const errors: string[] = [];
    const steps: ActionPlanPreview["steps"] = [];
    for (const id of actionIds) {
      const action = catalog.actions.find((a) => a.id === id);
      if (!action) {
        errors.push(`Unknown action: ${id}`);
        continue;
      }
      if (action.runner !== "local" || !action.command) {
        errors.push(`Action "${id}" is not a local runnable command`);
        continue;
      }
      steps.push({
        id: action.id,
        name: action.name,
        command: action.command,
        longRunning: action.longRunning,
        dependsOn: [],
      });
    }

    return { steps, errors, warnings: catalog.warnings };
  }

  private async runWorkflow(
    workflow: UnifiedWorkflow,
    catalog: ActionCatalog,
    options: ActionRunOptions & { cwd: string; project?: Project },
  ): Promise<ActionRunResult> {
    if (workflow.runner === "github") {
      const result = await runGitHubWorkflow(workflow, options.project);
      return {
        success: result.success,
        cwd: options.cwd,
        target: workflow.id,
        steps: [
          {
            name: workflow.name,
            actionId: workflow.id,
            status: result.success ? "success" : "failed",
            error: result.error,
            runner: "github",
          },
        ],
        remoteRunId: result.remoteRunId,
      };
    }

    if (workflow.runner === "n8n") {
      const result = await runN8nAction(workflow.id, options.project);
      return {
        success: result.success,
        cwd: options.cwd,
        target: workflow.id,
        steps: [
          {
            name: workflow.name,
            actionId: workflow.id,
            status: result.success ? "success" : "failed",
            error: result.error,
            runner: "n8n",
          },
        ],
        remoteRunId: result.remoteRunId,
      };
    }

    let actionIds: string[];
    try {
      actionIds = topoSortSteps(workflow.steps, options.only);
    } catch (e) {
      return {
        success: false,
        cwd: options.cwd,
        target: workflow.id,
        steps: [
          {
            name: workflow.name,
            actionId: workflow.id,
            status: "failed",
            error: e instanceof Error ? e.message : String(e),
            runner: "local",
          },
        ],
      };
    }

    const actions = actionIds.map((id) => {
      const action = catalog.actions.find((a) => a.id === id);
      if (!action) throw new Error(`Unknown action: ${id}`);
      return action;
    });

    return this.runActionList(actions, {
      ...options,
      target: workflow.id,
    });
  }

  private async runActionList(
    actions: UnifiedAction[],
    options: ActionRunOptions & {
      cwd: string;
      project?: Project;
      target: string | string[];
    },
  ): Promise<ActionRunResult> {
    const stopOnError = options.stopOnError ?? true;
    const results: ActionStepResult[] = [];
    let success = true;

    // Sync automation context builtins for any downstream that still needs them
    if (options.project) {
      const profile = profileFromProject(options.project as Project);
      createAutomationContext({
        cwd: options.cwd,
        variables: { ...profileVariables(profile), ...options.variables },
        secrets: options.secrets,
        profile,
      });
    }

    for (const action of actions) {
      if (!success && stopOnError) {
        results.push({
          name: action.name,
          actionId: action.id,
          command: action.command,
          status: "skipped",
          runner: action.runner,
        });
        continue;
      }

      if (action.runner === "github") {
        const remote = await runGitHubWorkflow(
          {
            id: action.id,
            name: action.name,
            source: "github",
            runner: "github",
            steps: [],
            remoteId: action.workflowId,
          },
          options.project,
        );
        results.push({
          name: action.name,
          actionId: action.id,
          status: remote.success ? "success" : "failed",
          error: remote.error,
          runner: "github",
        });
        if (!remote.success) {
          success = false;
          if (stopOnError) break;
        }
        continue;
      }

      if (action.runner === "n8n") {
        const remote = await runN8nAction(
          action.webhookId ?? action.id,
          options.project,
        );
        results.push({
          name: action.name,
          actionId: action.id,
          status: remote.success ? "success" : "failed",
          error: remote.error,
          runner: "n8n",
        });
        if (!remote.success) {
          success = false;
          if (stopOnError) break;
        }
        continue;
      }

      if (!action.command) {
        results.push({
          name: action.name,
          actionId: action.id,
          status: "failed",
          error: "Local action has no command",
          runner: "local",
        });
        success = false;
        if (stopOnError) break;
        continue;
      }

      try {
        // Ad-hoc Actions catalog entries are not `blocks` rows — omit blockId
        // so script_executions FK is satisfied (null is allowed for freestyle runs).
        const executionId = await scriptExecutionService.executeScript({
          command: action.command,
          workingDirectory: options.cwd,
        });
        const finished = await this.waitForExecution(
          executionId,
          action.longRunning,
        );
        const stepSuccess = finished.status === "success";
        results.push({
          name: action.name,
          actionId: action.id,
          command: action.command,
          status: stepSuccess ? "success" : "failed",
          executionId,
          exitCode: finished.exitCode,
          output: finished.output,
          error: stepSuccess
            ? undefined
            : summarizeExecutionFailure(finished),
          runner: "local",
        });
        if (!stepSuccess) {
          success = false;
          if (stopOnError) break;
        }
      } catch (error) {
        success = false;
        results.push({
          name: action.name,
          actionId: action.id,
          command: action.command,
          status: "failed",
          error: error instanceof Error ? error.message : String(error),
          runner: "local",
        });
        if (stopOnError) break;
      }
    }

    return {
      success,
      cwd: options.cwd,
      target: options.target,
      steps: results,
    };
  }

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

export const actionsService = ActionsService.getInstance();

/** Primary public API */
export const actions = {
  run: (
    target: string | string[],
    options?: ActionRunOptions & { project?: Project; cwd?: string },
  ) => actionsService.run(target, options),

  preview: (
    target: string | string[],
    options?: ActionRunOptions & { project?: Project; cwd?: string },
  ) => actionsService.preview(target, options),

  forProject: (project: Project) => actionsService.forProject(project),

  forDirectory: (cwd: string, packageManager?: string) => ({
    list: async () =>
      (await actionsService.catalogForDirectory(cwd, { packageManager }))
        .actions,
    workflows: async () =>
      (await actionsService.catalogForDirectory(cwd, { packageManager }))
        .workflows,
    catalog: () =>
      actionsService.catalogForDirectory(cwd, { packageManager }),
    run: (target: string | string[], options?: ActionRunOptions) =>
      actionsService.run(target, { ...options, cwd }),
    preview: (target: string | string[], options?: ActionRunOptions) =>
      actionsService.preview(target, { ...options, cwd }),
  }),

  catalogForProject: (project: Project, options?: ResolveCatalogOptions) =>
    actionsService.catalogForProject(project, options),
};
