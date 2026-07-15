/**
 * GitHub Actions catalog + dispatch adapter.
 */

import type { Project } from "$lib/domains/projects/types";
import { getProjectGitBranch } from "$lib/domains/projects/utils/display";
import { githubService } from "$lib/domains/github/service";
import { parseGitHubRemote } from "$lib/domains/github/utils/parseGitHubRemote";
import { invokeClient } from "$lib/utils/invokeClient";
import type {
  GitHubDispatchWorkflowRequest,
  GitHubListWorkflowsRequest,
  GitHubWorkflow,
} from "$lib/domains/github/types";
import type { UnifiedAction, UnifiedWorkflow } from "../types";

export interface AdapterCatalogSlice {
  actions: UnifiedAction[];
  workflows: UnifiedWorkflow[];
  warnings: string[];
}

export interface RemoteRunResult {
  success: boolean;
  remoteRunId?: string;
  error?: string;
}

async function resolveRepo(
  project: Project,
): Promise<{ owner: string; repo: string; branch?: string } | null> {
  try {
    const projectId = Number(project.id);
    if (!Number.isNaN(projectId)) {
      const link = await githubService.getProjectLink(projectId);
      if (link) {
        return {
          owner: link.repoOwner,
          repo: link.repoName,
          branch: link.defaultBranch ?? getProjectGitBranch(project),
        };
      }
    }
  } catch {
    // fall through to remote parse
  }

  const parsed = parseGitHubRemote(project.git_repository);
  if (!parsed) return null;
  return {
    owner: parsed.owner,
    repo: parsed.repo,
    branch: getProjectGitBranch(project),
  };
}

export async function listGitHubActions(
  project: Project,
): Promise<AdapterCatalogSlice> {
  const warnings: string[] = [];
  try {
      const status = await githubService.getConnectionStatus();
      if (!status.connected) {
        return { actions: [], workflows: [], warnings };
      }
  } catch {
    return { actions: [], workflows: [], warnings };
  }

  const repo = await resolveRepo(project);
  if (!repo) {
    return { actions: [], workflows: [], warnings };
  }

  try {
    const workflows = await invokeClient.post<GitHubWorkflow[]>(
      "github_list_workflows",
      {
        request: {
          owner: repo.owner,
          repo: repo.repo,
          perPage: 50,
        } satisfies GitHubListWorkflowsRequest,
      },
    );

    const unifiedWorkflows: UnifiedWorkflow[] = workflows
      .filter((w) => w.state === "active")
      .map((w) => ({
        id: `github:${w.id}`,
        name: w.name,
        description: w.path,
        source: "github" as const,
        runner: "github" as const,
        steps: [],
        remoteId: w.id,
      }));

    const actions: UnifiedAction[] = unifiedWorkflows.map((w) => ({
      id: w.id,
      name: w.name,
      description: w.description,
      source: "github",
      runner: "github",
      category: "ci",
      workflowId: typeof w.remoteId === "number" ? w.remoteId : Number(w.remoteId),
      workflowPath: w.description,
    }));

    return { actions, workflows: unifiedWorkflows, warnings };
  } catch (err) {
    warnings.push(
      `GitHub workflows unavailable: ${err instanceof Error ? err.message : String(err)}`,
    );
    return { actions: [], workflows: [], warnings };
  }
}

export async function runGitHubWorkflow(
  workflow: UnifiedWorkflow,
  project?: Project,
): Promise<RemoteRunResult> {
  if (!project) {
    return { success: false, error: "Project required to dispatch GitHub workflow" };
  }

  const repo = await resolveRepo(project);
  if (!repo) {
    return { success: false, error: "No GitHub repository linked to this project" };
  }

  const workflowId =
    typeof workflow.remoteId === "number"
      ? workflow.remoteId
      : Number(String(workflow.id).replace(/^github:/, ""));

  if (!workflowId || Number.isNaN(workflowId)) {
    return { success: false, error: `Invalid GitHub workflow id: ${workflow.id}` };
  }

  const refName = repo.branch || "main";

  try {
    await invokeClient.post<void>("github_dispatch_workflow", {
      request: {
        owner: repo.owner,
        repo: repo.repo,
        workflowId,
        refName,
      } satisfies GitHubDispatchWorkflowRequest,
    });
    return {
      success: true,
      remoteRunId: `${repo.owner}/${repo.repo}@${refName}`,
    };
  } catch (err) {
    return {
      success: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
