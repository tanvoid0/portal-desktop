/**
 * Turn a non-zero Action run into a structured, classified failure event — the
 * input the Pipeline-Doctor reasons about. Pure functions; no I/O.
 */

import type { ActionFailure, ActionFailureKind, ActionRunResult, ActionStepResult } from './types';

const OUTPUT_TAIL_MAX = 8 * 1024;

function tail(text: string, max = OUTPUT_TAIL_MAX): string {
  return text.length <= max ? text : text.slice(text.length - max);
}

/** The step that actually failed (prefer one with diagnostic detail). */
function failingStep(result: ActionRunResult): ActionStepResult | undefined {
  const failed = result.steps.filter((s) => s.status === 'failed');
  return failed.find((s) => s.error || s.output || s.exitCode != null) ?? failed[0];
}

interface Classification {
  kind: ActionFailureKind;
  hint?: string;
}

/**
 * Code-side, first-match-wins classification. Cheap, deterministic triage that
 * runs before any agent is spawned: a known `environment` failure never costs a
 * doctor run, it just shows a hint.
 */
export function classifyFailure(text: string, exitCode?: number | null): Classification {
  const t = text.toLowerCase();

  // Environment: the machine or its policy is wrong — not the pipeline, not the
  // app. No agent can fix this from inside the repo (see QA-129).
  if (
    t.includes('lockfile contains entries that the active policies reject') ||
    t.includes('confirmmodulespurge') ||
    t.includes('err_pnpm_lockfile_config_mismatch')
  ) {
    return {
      kind: 'environment',
      hint: "pnpm's lockfile policy rejected this project. Switch the package manager to npm for this project, or run `pnpm clean --lockfile` then `pnpm install`.",
    };
  }

  // Pipeline: the command itself is wrong or missing — the action is mapped to a
  // script that doesn't exist. exit 127 (POSIX) / 9009 (Windows) == "not found".
  if (
    /err_pnpm_no_script|missing script|command not found|is not recognized as an internal or external command|no such file or directory/i.test(
      text
    ) ||
    exitCode === 127 ||
    exitCode === 9009
  ) {
    return {
      kind: 'pipeline',
      hint: 'The command does not exist in this project — the action is likely mapped to the wrong script.',
    };
  }

  // Everything else (test/lint/build failures, stack traces) is probably app
  // code, or unclassifiable — let an agent decide.
  return { kind: 'unknown' };
}

/** Build a failure event from a run result, or null when the run succeeded. */
export function toFailure(result: ActionRunResult): ActionFailure | null {
  if (result.success) return null;
  const step = failingStep(result);
  if (!step) return null;

  const outputTail = tail([step.output, step.error].filter(Boolean).join('\n').trim());
  const { kind, hint } = classifyFailure(outputTail, step.exitCode);

  return {
    executionId: step.executionId,
    actionId: step.actionId,
    runner: step.runner,
    command: step.command,
    exitCode: step.exitCode,
    outputTail,
    cwd: result.cwd,
    target: result.target,
    kind,
    hint,
  };
}
