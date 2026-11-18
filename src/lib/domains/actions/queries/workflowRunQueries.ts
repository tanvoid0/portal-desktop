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
      refetchInterval: (q) => {
        const rows = q.state.data;
        if (!rows) return false;
        if (rows.some(isActive)) return 1000;
        // Between steps: a planned step has no row yet and the run hasn't failed/stopped.
        const plan = rows.find((r) => r.workflowPlan)?.workflowPlan ?? [];
        const over = rows.some((r) => r.status === 'failed' || r.status === 'cancelled');
        const started = new Set(rows.map((r) => r.stepId));
        return !over && plan.some((s) => !started.has(s.id)) ? 1000 : false;
      },
    };
  });
}
