export const queryKeys = {
  dashboard: {
    overview: ["dashboard", "overview"] as const,
  },
  projects: {
    all: ["projects"] as const,
    detail: (id: number | string) => ["projects", id] as const,
  },
  github: {
    status: ["github", "status"] as const,
    repositories: (search: string) => ["github", "repositories", search] as const,
    linkedRepos: ["github", "linked-repos"] as const,
    repository: (owner: string, repo: string) =>
      ["github", "repository", owner, repo] as const,
    issues: (scope: string) => ["github", "issues", scope] as const,
    workflowRuns: (owner: string, repo: string, scope: string) =>
      ["github", "workflow-runs", owner, repo, scope] as const,
    workflowRun: (owner: string, repo: string, runId: number) =>
      ["github", "workflow-run", owner, repo, runId] as const,
    workflowJobLogs: (owner: string, repo: string, jobId: number) =>
      ["github", "workflow-job-logs", owner, repo, jobId] as const,
    projectLink: (projectId: number | string) =>
      ["github", "project-link", projectId] as const,
  },
  tasks: {
    all: ["tasks"] as const,
    detail: (id: string) => ["tasks", id] as const,
  },
  cloud: {
    resources: (type: string, namespace: string) =>
      ["cloud", type, namespace] as const,
  },
  sdk: {
    managers: ["sdk", "managers"] as const,
    versions: (name: string) => ["sdk", name, "versions"] as const,
  },
};
