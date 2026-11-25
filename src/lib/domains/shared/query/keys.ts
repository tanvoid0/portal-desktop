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
  actions: {
    workflowRun: (runId: string) => ['actions', 'workflow-run', runId] as const,
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
    platformStatus: ['ai', 'platform', 'status'] as const,
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
    events: ['agents', 'events'] as const,
    feed: (backendId: string) => ['agents', 'events', backendId] as const,
    analytics: (backendId: string) => ['agents', 'analytics', backendId] as const,
  },
  sync: {
    all: ['sync'] as const,
    config: ['sync', 'config'] as const,
    status: ['sync', 'status'] as const,
    versions: ['sync', 'versions'] as const,
    unresolved: ['sync', 'unresolved'] as const,
  },
  database: {
    sources: ['database', 'sources'] as const,
    tables: (sourceId: string) => ['database', sourceId, 'tables'] as const,
    columns: (sourceId: string, table: string) => ['database', sourceId, table, 'columns'] as const,
    rows: (sourceId: string, table: string, page: number) =>
      ['database', sourceId, table, 'rows', page] as const,
    mongoDatabases: (sourceId: string) => ['database', sourceId, 'mongo', 'dbs'] as const,
    mongoCollections: (sourceId: string, db: string) =>
      ['database', sourceId, 'mongo', db, 'collections'] as const,
    // Prefix of every mongoDocs key for one collection (invalidate after a write).
    mongoCollection: (sourceId: string, db: string, coll: string) =>
      ['database', sourceId, 'mongo', db, coll] as const,
    // `filter` is the applied filter text; `page` the 0-based page.
    mongoDocs: (sourceId: string, db: string, coll: string, filter: string, page: number) =>
      ['database', sourceId, 'mongo', db, coll, 'docs', filter, page] as const,
    redisDatabases: (sourceId: string) => ['database', sourceId, 'redis', 'dbs'] as const,
    // Prefix of redisKeys / redisValue for one logical db (invalidate after a write).
    redisDb: (sourceId: string, db: number) => ['database', sourceId, 'redis', 'db', db] as const,
    redisKeys: (sourceId: string, db: number, pattern: string) =>
      ['database', sourceId, 'redis', 'db', db, 'keys', pattern] as const,
    redisValue: (sourceId: string, db: number, key: string) =>
      ['database', sourceId, 'redis', 'db', db, 'value', key] as const,
  },
  devices: {
    all: ['devices'] as const,
    capabilities: ['devices', 'capabilities'] as const,
    lanEnabled: ['devices', 'lan-enabled'] as const,
    connectivity: ['devices', 'connectivity'] as const,
  },
  disk: {
    hostStats: ['disk', 'host-stats'] as const,
    usage: ['disk', 'usage'] as const,
  },
};
