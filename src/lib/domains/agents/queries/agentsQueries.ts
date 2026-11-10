import { queryKeys } from '$lib/domains/shared/query/keys';
import { agentsApi } from '../api/agentsApi.js';

/** Detected agent backends (Claude Code today) and their reported models. */
export const agentBackendsQuery = () => ({
  queryKey: queryKeys.agents.backends,
  queryFn: () => agentsApi.listBackends(),
});
