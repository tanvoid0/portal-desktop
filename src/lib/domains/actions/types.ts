/**
 * Unified Actions model — one catalog for local, GitHub, and n8n runners.
 */

export type ActionSource = 'local' | 'github' | 'n8n' | 'file' | 'custom';
export type ActionRunner = 'local' | 'github' | 'n8n';
export type ActionCategory =
  'install' | 'dev' | 'build' | 'test' | 'lint' | 'utility' | 'ci' | 'deploy';

export interface UnifiedAction {
  id: string;
  name: string;
  description?: string;
  source: ActionSource;
  runner: ActionRunner;
  category?: ActionCategory | string;
  /** Resolved shell command (local runner) */
  command?: string;
  longRunning?: boolean;
  /** GitHub Actions workflow id */
  workflowId?: number;
  workflowPath?: string;
  /** n8n webhook / workflow id */
  webhookId?: string;
  framework?: string;
}

export interface UnifiedWorkflowStep {
  action: string;
  needs?: string[];
}

export interface UnifiedWorkflow {
  id: string;
  name: string;
  description?: string;
  source: ActionSource;
  runner: ActionRunner;
  steps: UnifiedWorkflowStep[];
  /** For GitHub / n8n, the remote id */
  remoteId?: string | number;
}

export interface ActionRunOptions {
  cwd?: string;
  variables?: Record<string, string | number | boolean>;
  secrets?: Record<string, string>;
  stopOnError?: boolean;
  /** Filter workflow steps to these action ids */
  only?: string[];
  projectId?: string;
  /**
   * Fired the moment a local step's process is spawned (before it finishes), so
   * the UI can surface the run as "running" with live output instead of waiting
   * for the whole run to resolve. See QA-130.
   */
  onStepStart?: (step: { actionId: string; executionId: string }) => void;
}

export interface ActionStepResult {
  name: string;
  actionId: string;
  command?: string;
  status: 'success' | 'failed' | 'skipped';
  executionId?: string;
  exitCode?: number | null;
  output?: string;
  error?: string;
  runner: ActionRunner;
}

export interface ActionRunResult {
  success: boolean;
  cwd: string;
  /** What was requested: action id, list, or workflow id */
  target: string | string[];
  steps: ActionStepResult[];
  /** Remote run id when github/n8n */
  remoteRunId?: string;
  /** Structured failure signal for a non-zero local run (the Pipeline-Doctor's input). */
  failure?: ActionFailure;
}

/**
 * How a failure is classified, which decides what the doctor does next:
 * - `pipeline`: the command is wrong/missing (fix the action definition).
 * - `environment`: the machine/policy is wrong (e.g. pnpm lockfile policy) — the
 *   app and the pipeline are fine; the user needs an env/PM change.
 * - `unknown`: likely an app-code failure, or something we can't classify — hand
 *   to an agent to decide.
 */
export type ActionFailureKind = 'pipeline' | 'environment' | 'unknown';

/** Structured, queryable failure event emitted on a non-zero local Action run. */
export interface ActionFailure {
  /** script_executions row id — the durable record of this failure. */
  executionId?: string;
  actionId: string;
  runner: ActionRunner;
  command?: string;
  exitCode?: number | null;
  /** Last ~8 KB of combined stdout/stderr, enough for diagnosis without bloat. */
  outputTail: string;
  cwd: string;
  target: string | string[];
  kind: ActionFailureKind;
  /** One-line, user-facing hint when the failure class is recognised. */
  hint?: string;
}

export interface ActionPlanPreview {
  ready: boolean;
  steps: Array<{
    id: string;
    name: string;
    command: string;
    longRunning?: boolean;
    dependsOn: string[];
  }>;
  errors: string[];
  warnings: string[];
}

/** Declarative `.portal/pipeline.yml` schema (version 1) */
export interface PortalPipelineFile {
  version: 1;
  release?: ReleaseSpec;
  actions?: Record<
    string,
    {
      name?: string;
      description?: string;
      run: string;
      /** Relative to the project root. */
      cwd?: string;
      env?: Record<string, string>;
      category?: string;
      longRunning?: boolean;
    }
  >;
  workflows?: Record<
    string,
    {
      name?: string;
      description?: string;
      steps: Array<{
        action: string;
        needs?: string[];
      }>;
    }
  >;
}

/** Resolved project context for command templates */
export interface ProjectAutomationProfile {
  id?: string;
  name: string;
  path: string;
  packageManager: string;
  framework?: string;
  frameworks: string[];
  buildCommand?: string;
  startCommand?: string;
  testCommand?: string;
  /**
   * package.json `scripts`. `undefined` = not probed (assume all exist);
   * `null` = no package.json at the project root.
   */
  scripts?: Record<string, string> | null;
}

/** `release:` block of `.portal/pipeline.yml`: which workflow ships, which rehearses. */
export interface ReleaseSpec {
  kind?: string;
  ship: string;
  rehearse?: string;
  bump?: string;
  env?: Record<string, string>;
}

/** Result of the `release_describe` command. */
export interface ReleaseDescription {
  release: ReleaseSpec;
  ship: { id: string; steps: { id: string; name: string; dependsOn: string[]; command: string }[] };
  rehearse: { id: string; steps: { id: string; name: string }[] } | null;
}

/** Item from `release_list`. */
export interface ReleaseRun {
  id: string;
  status: string;
  startedAt: string;
  pipelineName: string;
}
