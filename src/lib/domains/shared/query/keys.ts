export const queryKeys = {
  dashboard: {
    overview: ['dashboard', 'overview'] as const,
  },
  projects: {
    all: ['projects'] as const,
    detail: (id: number | string) => ['projects', id] as const,
    modules: ['project-modules'] as const,
    /** Under `projects` so any project write refetches it. */
    unavailable: ['projects', 'unavailable'] as const,
    env: (id: number) => ['projects', id, 'env'] as const,
    moves: ['projects', 'moves'] as const,
  },
  github: {
    status: ['github', 'status'] as const,
    repositories: (search: string) => ['github', 'repositories', search] as const,
    linkedRepos: ['github', 'linked-repos'] as const,
    repository: (owner: string, repo: string) => ['github', 'repository', owner, repo] as const,
    issues: (scope: string) => ['github', 'issues', scope] as const,
    workflowRuns: (owner: string, repo: string, scope: string) =>
      ['github', 'workflow-runs', owner, repo, scope] as const,
    workflowRun: (owner: string, repo: string, runId: number) =>
      ['github', 'workflow-run', owner, repo, runId] as const,
    workflowJobLogs: (owner: string, repo: string, jobId: number) =>
      ['github', 'workflow-job-logs', owner, repo, jobId] as const,
    projectLink: (projectId: number | string) => ['github', 'project-link', projectId] as const,
  },
  ai: {
    logs: (filters: object) => ['ai', 'logs', filters] as const,
  },
  tasks: {
    all: ['tasks'] as const,
    detail: (id: string) => ['tasks', id] as const,
  },
  credentials: {
    status: ['credentials', 'vault-status'] as const,
  },
  cloud: {
    resources: (type: string, namespace: string) => ['cloud', type, namespace] as const,
  },
  sdk: {
    configs: ['sdk', 'configs'] as const,
    managers: ['sdk', 'managers'] as const,
    versions: (name: string) => ['sdk', name, 'versions'] as const,
  },
  agents: {
    backends: ['agents', 'backends'] as const,
    sessions: ['agents', 'sessions'] as const,
    session: (id: string) => ['agents', 'sessions', id] as const,
  },
  devices: {
    all: ['devices'] as const,
    capabilities: ['devices', 'capabilities'] as const,
    lanEnabled: ['devices', 'lan-enabled'] as const,
    connectivity: ['devices', 'connectivity'] as const,
  },
};
