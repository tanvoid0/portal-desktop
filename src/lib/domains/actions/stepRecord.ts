import type { ActionStepResult } from './types';
import type { WorkflowPlanStep } from '$lib/domains/scripts/services/scriptExecutionService';

export interface RunLink {
  workflowRunId: string;
  workflowId: string;
  workflowPlan: WorkflowPlanStep[];
}

/** Payload for `recordInteractive`: a finished row for a workflow step that spawned no process. */
export function stepRecordPayload(run: RunLink, step: ActionStepResult) {
  return {
    command: step.command ?? step.name,
    exitCode: step.status === 'success' ? 0 : null,
    status: step.status,
    error: step.error,
    workflowRunId: run.workflowRunId,
    workflowId: run.workflowId,
    stepId: step.actionId,
    workflowPlan: run.workflowPlan,
  };
}
