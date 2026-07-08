import { invokeClient } from "$lib/utils/invokeClient";
import { logger } from "$lib/domains/shared/services/logger";
import { normalizeProject } from "$lib/domains/projects/utils/normalizeProject";
import { queryClient } from "$lib/domains/shared/query";
import { invalidateProjectsList } from "$lib/domains/shared/query/invalidateProjects";
import { queryKeys } from "$lib/domains/shared/query/keys";
import type {
  GitHubCloneRepositoryRequest,
  GitHubConnectionStatus,
  GitHubCreateIssueRequest,
  GitHubDeviceFlowPollResult,
  GitHubDeviceFlowStart,
  GitHubIssue,
  GitHubLinkExistingRepositoryRequest,
  GitHubListIssuesRequest,
  GitHubLocalRepositoryDetection,
  GitHubProjectLink,
  GitHubProjectLinkResult,
  GitHubRepoProjects,
  GitHubRepository,
  GitHubUpdateIssueRequest,
} from "./types";

const log = logger.createScoped("GitHubService");

class GitHubService {
  async getConnectionStatus(): Promise<GitHubConnectionStatus> {
    return invokeClient.post<GitHubConnectionStatus>("github_get_connection_status");
  }

  async startDeviceFlow(scope?: string): Promise<GitHubDeviceFlowStart> {
    return invokeClient.post<GitHubDeviceFlowStart>("github_start_device_flow", {
      scope,
    });
  }

  async pollDeviceFlow(deviceCode: string): Promise<GitHubDeviceFlowPollResult> {
    return invokeClient.post<GitHubDeviceFlowPollResult>("github_poll_device_flow", {
      deviceCode,
    });
  }

  async connectWithDeviceFlow(scope?: string): Promise<GitHubConnectionStatus> {
    const started = await this.startDeviceFlow(scope);
    const target = started.verificationUriComplete || started.verificationUri;
    window.open(target, "_blank", "noopener,noreferrer");

    const startedAt = Date.now();
    while (Date.now() - startedAt < started.expiresIn * 1000) {
      const polled = await this.pollDeviceFlow(started.deviceCode);
      if (polled.status === "connected") {
        await this.invalidateGitHubCaches();
        return this.getConnectionStatus();
      }
      if (polled.status === "expired" || polled.status === "denied") {
        throw new Error(polled.message || `GitHub login ${polled.status}`);
      }
      if (polled.status === "error") {
        throw new Error(polled.message || "GitHub login failed");
      }
      const delayMs = (polled.retryAfterSeconds || started.interval || 5) * 1000;
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }

    throw new Error("GitHub device flow expired before authorization completed");
  }

  async disconnect(): Promise<GitHubConnectionStatus> {
    const status = await invokeClient.post<GitHubConnectionStatus>("github_disconnect");
    await this.invalidateGitHubCaches();
    return status;
  }

  async listRepositories(search = "", page = 1, perPage = 50): Promise<GitHubRepository[]> {
    return invokeClient.post<GitHubRepository[]>("github_list_repositories", {
      search,
      page,
      perPage,
    });
  }

  async getRepository(owner: string, repo: string): Promise<GitHubRepoProjects> {
    const raw = await invokeClient.post<GitHubRepoProjects>("github_get_repository", {
      owner,
      repo,
    });
    return {
      ...raw,
      linkedProjects: (raw.linkedProjects || []).map((project) =>
        normalizeProject(project as unknown as Record<string, unknown>),
      ),
    };
  }

  async listIssues(request: GitHubListIssuesRequest): Promise<GitHubIssue[]> {
    return invokeClient.post<GitHubIssue[]>("github_list_issues", { request });
  }

  async getIssue(owner: string, repo: string, number: number): Promise<GitHubIssue> {
    return invokeClient.post<GitHubIssue>("github_get_issue", {
      owner,
      repo,
      number,
    });
  }

  async createIssue(request: GitHubCreateIssueRequest): Promise<GitHubIssue> {
    const issue = await invokeClient.post<GitHubIssue>("github_create_issue", {
      request,
    });
    await this.invalidateGitHubCaches();
    return issue;
  }

  async updateIssue(request: GitHubUpdateIssueRequest): Promise<GitHubIssue> {
    const issue = await invokeClient.post<GitHubIssue>("github_update_issue", {
      request,
    });
    await this.invalidateGitHubCaches();
    return issue;
  }

  async getProjectLink(projectId: number): Promise<GitHubProjectLink | null> {
    return invokeClient.post<GitHubProjectLink | null>("github_get_project_link", {
      projectId,
    });
  }

  async detectLocalRepository(
    path: string,
  ): Promise<GitHubLocalRepositoryDetection> {
    return invokeClient.post<GitHubLocalRepositoryDetection>(
      "github_detect_local_repository",
      { path },
    );
  }

  async cloneRepository(
    request: GitHubCloneRepositoryRequest,
  ): Promise<GitHubProjectLinkResult> {
    log.info("Cloning GitHub repository", request);
    const raw = await invokeClient.post<
      Omit<GitHubProjectLinkResult, "project"> & { project: Record<string, unknown> }
    >("github_clone_repository", request);
    invalidateProjectsList(queryClient);
    await this.invalidateGitHubCaches();
    return {
      ...raw,
      project: normalizeProject(raw.project),
    };
  }

  async linkExistingRepository(
    request: GitHubLinkExistingRepositoryRequest,
  ): Promise<GitHubProjectLinkResult> {
    log.info("Linking local repository to GitHub", request);
    const raw = await invokeClient.post<
      Omit<GitHubProjectLinkResult, "project"> & { project: Record<string, unknown> }
    >("github_link_existing_repository", { request });
    invalidateProjectsList(queryClient);
    await this.invalidateGitHubCaches();
    return {
      ...raw,
      project: normalizeProject(raw.project),
    };
  }

  async invalidateGitHubCaches(): Promise<void> {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.github.status }),
      queryClient.invalidateQueries({ queryKey: ["github"] }),
    ]);
  }
}

export const githubService = new GitHubService();
