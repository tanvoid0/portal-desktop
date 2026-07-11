import { createQuery } from "@tanstack/svelte-query";
import { queryKeys } from "$lib/domains/shared/query/keys";
import { githubService } from "./service";

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
