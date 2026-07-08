import { invokeClient } from "$lib/utils/invokeClient";
import { openExternalUrl } from "$lib/utils/tauri";
import { logger } from "$lib/domains/shared/services/logger";
import { normalizeProject } from "$lib/domains/projects/utils/normalizeProject";
import { queryClient } from "$lib/domains/shared/query";
import { invalidateProjectsList } from "$lib/domains/shared/query/invalidateProjects";
import { queryKeys } from "$lib/domains/shared/query/keys";
import type {
  GitHubCloneRepositoryRequest,
  GitHubConnectionStatus,
  GitHubCreateIssueRequest,
  GitHubDeviceFlowCallbacks,
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

const DEVICE_FLOW_TIMEOUT_MS = 30_000;

function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  label: string,
): Promise<T> {
  return Promise.race([
    promise,
    new Promise<never>((_, reject) => {
      setTimeout(
        () => reject(new Error(`${label} timed out after ${timeoutMs}ms`)),
        timeoutMs,
      );
    }),
  ]);
}

function isPollResult(value: unknown): value is GitHubDeviceFlowPollResult {
  return (
    typeof value === "object" &&
    value !== null &&
    "status" in value &&
    typeof (value as GitHubDeviceFlowPollResult).status === "string"
  );
}

function formatDeviceFlowError(error: unknown): Error {
  const message = error instanceof Error ? error.message : String(error);
  if (
    message.includes("Failed to fetch") ||
    message.includes("CONNECTION_REFUSED") ||
    message.includes("Tauri environment required")
  ) {
    return new Error(
      "Lost connection to the desktop app backend. Restart Portal Desktop and try again.",
    );
  }
  return new Error(`GitHub authorization check failed: ${message}`);
}

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

  async connectWithDeviceFlow(
    scope?: string,
    callbacks?: GitHubDeviceFlowCallbacks,
  ): Promise<GitHubConnectionStatus> {
    const started = await withTimeout(
      this.startDeviceFlow(scope),
      DEVICE_FLOW_TIMEOUT_MS,
      "GitHub device flow start",
    );

    await callbacks?.onStarted?.(started);

    const target = started.verificationUriComplete || started.verificationUri;
    try {
      await openExternalUrl(target);
    } catch (error) {
      log.warn("Failed to open GitHub authorization URL automatically", error);
    }

    const startedAt = Date.now();
    while (Date.now() - startedAt < started.expiresIn * 1000) {
      await callbacks?.onPolling?.();

      let polled: GitHubDeviceFlowPollResult;
      try {
        polled = await withTimeout(
          this.pollDeviceFlow(started.deviceCode),
          DEVICE_FLOW_TIMEOUT_MS,
          "GitHub authorization check",
        );
      } catch (error) {
        throw formatDeviceFlowError(error);
      }

      if (!isPollResult(polled)) {
        throw new Error("Invalid response while checking GitHub authorization");
      }

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
      if (polled.status !== "pending") {
        throw new Error(
          polled.message || `Unexpected GitHub login status: ${polled.status}`,
        );
      }

      const delayMs =
        (polled.retryAfterSeconds || started.interval || 5) * 1000;
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
