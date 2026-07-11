import { createQuery } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";
import { githubService } from "./service";
import { isWorkflowJobActive, isWorkflowRunActive } from "./utils/workflowDisplay";

const WORKFLOW_POLL_INTERVAL_MS = 3_000;

export function createGitHubStatusQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.github.status,
    queryFn: () => githubService.getConnectionStatus(),
  }));
}

export function createGitHubRepositoriesQuery(
  search: () => string,
  enabled: () => boolean = () => true,
  page: () => number = () => 1,
  perPage: () => number = () => 50,
) {
  return createQuery(() => ({
    queryKey: queryKeys.github.repositories(search()),
    enabled: enabled(),
    queryFn: () => githubService.listRepositories(search(), page(), perPage()),
  }));
}

export function createGitHubLinkedRepositoriesQuery(
  enabled: () => boolean = () => true,
) {
  return createQuery(() => ({
    queryKey: queryKeys.github.linkedRepos,
    enabled: enabled(),
    queryFn: () => githubService.listLinkedRepositories(),
  }));
}

export function createGitHubRepositoryQuery(
  owner: () => string | undefined,
  repo: () => string | undefined,
  enabled: () => boolean = () => true,
) {
  return createQuery(() => {
    const ownerValue = owner();
    const repoValue = repo();
    return {
      queryKey: queryKeys.github.repository(ownerValue ?? "", repoValue ?? ""),
      enabled: enabled() && Boolean(ownerValue && repoValue),
      queryFn: () => githubService.getRepository(ownerValue!, repoValue!),
    };
  });
}

export function createGitHubIssuesQuery(
  scope: () => {
    owner?: string;
    repo?: string;
    state?: string;
    filter?: string;
    page?: number;
    perPage?: number;
    includePullRequests?: boolean;
  },
  enabled: () => boolean = () => true,
) {
  return createQuery(() => {
    const request = scope();
    const scopeKey = JSON.stringify(request);
    return {
      queryKey: queryKeys.github.issues(scopeKey),
      enabled: enabled(),
      queryFn: () => githubService.listIssues(request),
    };
  });
}

export function createGitHubWorkflowRunsQuery(
  scope: () => {
    owner: string;
    repo: string;
    branch?: string;
    status?: string;
    page?: number;
    perPage?: number;
  },
  enabled: () => boolean = () => true,
) {
  return createQuery(() => {
    const request = scope();
    const scopeKey = JSON.stringify(request);
    return {
      queryKey: queryKeys.github.workflowRuns(
        request.owner,
        request.repo,
        scopeKey,
      ),
      enabled: enabled() && Boolean(request.owner && request.repo),
      queryFn: () => githubService.listWorkflowRuns(request),
      refetchInterval: (query) => {
        const runs = query.state.data;
        if (!runs?.some(isWorkflowRunActive)) return false;
        return WORKFLOW_POLL_INTERVAL_MS;
      },
    };
  });
}

export function createGitHubWorkflowRunQuery(
  owner: () => string | undefined,
  repo: () => string | undefined,
  runId: () => number | undefined,
  enabled: () => boolean = () => true,
) {
  return createQuery(() => {
    const ownerValue = owner();
    const repoValue = repo();
    const runIdValue = runId();
    return {
      queryKey: queryKeys.github.workflowRun(
        ownerValue ?? "",
        repoValue ?? "",
        runIdValue ?? 0,
      ),
      enabled:
        enabled() && Boolean(ownerValue && repoValue && runIdValue != null),
      queryFn: () =>
        githubService.getWorkflowRun(ownerValue!, repoValue!, runIdValue!),
      refetchInterval: (query) => {
        const detail = query.state.data;
        if (!detail) return false;
        if (isWorkflowRunActive(detail.run)) return WORKFLOW_POLL_INTERVAL_MS;
        if (detail.jobs.some((job) => isWorkflowJobActive(job.status))) {
          return WORKFLOW_POLL_INTERVAL_MS;
        }
        return false;
      },
    };
  });
}

export function createGitHubWorkflowJobLogsQuery(
  owner: () => string | undefined,
  repo: () => string | undefined,
  jobId: () => number | undefined,
  enabled: () => boolean = () => true,
) {
  return createQuery(() => {
    const ownerValue = owner();
    const repoValue = repo();
    const jobIdValue = jobId();
    return {
      queryKey: queryKeys.github.workflowJobLogs(
        ownerValue ?? "",
        repoValue ?? "",
        jobIdValue ?? 0,
      ),
      enabled:
        enabled() && Boolean(ownerValue && repoValue && jobIdValue != null),
      queryFn: () =>
        githubService.getWorkflowJobLogs(ownerValue!, repoValue!, jobIdValue!),
    };
  });
}
