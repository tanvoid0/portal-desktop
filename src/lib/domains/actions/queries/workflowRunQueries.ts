import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { scriptExecutionService } from '$lib/domains/scripts/services/scriptExecutionService';
import { isActive } from '../runActions';

/** Rows of one workflow run; polls every second while any step is active. */
export function createWorkflowRunQuery(runId: () => string | null | undefined) {
  return createQuery(() => {
    const id = runId();
    return {
      queryKey: queryKeys.actions.workflowRun(id ?? ''),
      queryFn: () => scriptExecutionService.getByWorkflowRun(id!),
      enabled: !!id,
      refetchInterval: (q) => (q.state.data?.some(isActive) ? 1000 : false),
    };
  });
}
