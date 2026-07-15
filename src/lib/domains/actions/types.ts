/**
 * Unified Actions model — one catalog for local, GitHub, and n8n runners.
 */

export type ActionSource = "local" | "github" | "n8n" | "file" | "custom";
export type ActionRunner = "local" | "github" | "n8n";
export type ActionCategory =
  | "install"
  | "dev"
  | "build"
  | "test"
  | "lint"
  | "utility"
  | "ci"
  | "deploy";

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
}

export interface ActionStepResult {
  name: string;
  actionId: string;
  command?: string;
  status: "success" | "failed" | "skipped";
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
  actions?: Record<
    string,
    {
      name?: string;
      description?: string;
      run: string;
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
}
