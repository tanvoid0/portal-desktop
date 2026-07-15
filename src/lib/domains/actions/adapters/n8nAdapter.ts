/**
 * n8n workflow catalog + trigger adapter.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Project } from "$lib/domains/projects/types";
import {
  getProjectFramework,
  getProjectPackageManager,
} from "$lib/domains/projects/utils/display";
import { resolvePackageManagerName } from "../profile";
import type { AvailableWorkflow, WorkflowResult } from "$lib/domains/automation/types";
import type { UnifiedAction, UnifiedWorkflow } from "../types";
import type { AdapterCatalogSlice, RemoteRunResult } from "./githubAdapter";

export async function listN8nActions(
  project: Project,
): Promise<AdapterCatalogSlice> {
  const warnings: string[] = [];

  try {
    const healthy = await invoke<boolean>("check_n8n_health");
    if (!healthy) {
      return { actions: [], workflows: [], warnings };
    }
  } catch {
    return { actions: [], workflows: [], warnings };
  }

  try {
    const framework = getProjectFramework(project);
    const packageManager =
      resolvePackageManagerName(project) || getProjectPackageManager(project);

    let workflows: AvailableWorkflow[] = [];
    try {
      workflows = await invoke<AvailableWorkflow[]>("get_suggested_workflows", {
        framework,
        packageManager,
      });
    } catch {
      workflows = await invoke<AvailableWorkflow[]>("list_available_workflows");
    }

    const unifiedWorkflows: UnifiedWorkflow[] = workflows.map((w) => ({
      id: `n8n:${w.id}`,
      name: w.name,
      description: w.description,
      source: "n8n" as const,
      runner: "n8n" as const,
      steps: [],
      remoteId: w.id,
    }));

    const actions: UnifiedAction[] = unifiedWorkflows.map((w) => ({
      id: w.id,
      name: w.name,
      description: w.description,
      source: "n8n",
      runner: "n8n",
      category: "utility",
      webhookId: String(w.remoteId ?? w.id.replace(/^n8n:/, "")),
      framework: workflows.find((x) => `n8n:${x.id}` === w.id)?.framework,
    }));

    return { actions, workflows: unifiedWorkflows, warnings };
  } catch (err) {
    warnings.push(
      `n8n workflows unavailable: ${err instanceof Error ? err.message : String(err)}`,
    );
    return { actions: [], workflows: [], warnings };
  }
}

export async function runN8nAction(
  workflowIdOrPrefixed: string,
  project?: Project,
): Promise<RemoteRunResult> {
  if (!project) {
    return { success: false, error: "Project required to trigger n8n workflow" };
  }

  const workflowId = workflowIdOrPrefixed.replace(/^n8n:/, "");

  try {
    const result = await invoke<WorkflowResult>("trigger_n8n_workflow", {
      workflowId,
      projectData: {
        id: project.id,
        name: project.name,
        path: project.path,
        framework: getProjectFramework(project),
        package_manager: resolvePackageManagerName(project),
        build_command: project.build_command,
        start_command: project.start_command,
        test_command: project.test_command,
      },
    });

    return {
      success: result.success,
      remoteRunId: result.execution_id,
      error: result.success
        ? undefined
        : result.errors?.join("; ") || "n8n workflow failed",
    };
  } catch (err) {
    return {
      success: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
