import type {
  ScriptExecutionInfo,
  WorkflowPlanStep,
} from '$lib/domains/scripts/services/scriptExecutionService';

export type StepStatus = ScriptExecutionInfo['status'] | 'skipped';

export interface StepNode {
  id: string;
  name: string;
  status: StepStatus;
  /** The step's run, when it has started. */
  executionId?: string;
}

export interface StepGraph {
  columns: StepNode[][];
  edges: { from: string; to: string }[];
}

/** One step per id (first wins, `needs` merged) — duplicate ids would duplicate node keys. */
export function dedupePlan(plan: WorkflowPlanStep[]): WorkflowPlanStep[] {
  const byId = new Map<string, WorkflowPlanStep>();
  for (const s of plan) {
    const first = byId.get(s.id);
    if (first) first.needs = [...new Set([...first.needs, ...s.needs])];
    else byId.set(s.id, { ...s });
  }
  return [...byId.values()];
}

/** Stage = longest path from a root. A cycle or unknown `needs` collapses to one column. */
export function buildStepGraph(
  rawPlan: WorkflowPlanStep[],
  executions: ScriptExecutionInfo[] = []
): StepGraph {
  const plan = dedupePlan(rawPlan);
  const rows = new Map(executions.filter((e) => e.stepId).map((e) => [e.stepId!, e]));
  // A run is over once a step failed or was stopped; steps with no row then never ran.
  // ponytail: a clean run has no gap to detect, so only failed/cancelled count as "over".
  const over = executions.some((e) => e.status === 'failed' || e.status === 'cancelled');
  const ids = new Set(plan.map((s) => s.id));
  const node = (s: WorkflowPlanStep): StepNode => {
    const row = rows.get(s.id);
    return {
      id: s.id,
      name: s.name,
      status: row?.status ?? (over ? 'skipped' : 'pending'),
      executionId: row?.id,
    };
  };

  const stage = new Map<string, number>();
  const visiting = new Set<string>();
  let broken = false;
  const depth = (s: WorkflowPlanStep): number => {
    const known = stage.get(s.id);
    if (known !== undefined) return known;
    if (visiting.has(s.id)) {
      broken = true;
      return 0;
    }
    visiting.add(s.id);
    let d = 0;
    for (const n of s.needs) {
      const dep = plan.find((p) => p.id === n);
      if (!dep) broken = true;
      else d = Math.max(d, depth(dep) + 1);
    }
    visiting.delete(s.id);
    stage.set(s.id, d);
    return d;
  };
  plan.forEach(depth);

  const edges = broken
    ? []
    : plan.flatMap((s) => s.needs.filter((n) => ids.has(n)).map((n) => ({ from: n, to: s.id })));
  const columns: StepNode[][] = [];
  for (const s of plan) (columns[broken ? 0 : stage.get(s.id)!] ??= []).push(node(s));
  return { columns, edges };
}

/** The execution to show once a run ends: the first failed/cancelled step, else the last one run. */
export function pickRunSelection(
  steps: { status: string; executionId?: string }[]
): string | undefined {
  const ran = steps.filter((s) => s.executionId);
  return (ran.find((s) => s.status === 'failed' || s.status === 'cancelled') ?? ran.at(-1))
    ?.executionId;
}
