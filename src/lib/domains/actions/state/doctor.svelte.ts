import type { Project } from '$lib/domains/projects/types';
import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
import { actions } from '../actionsService';
import { buildDiagnosePrompt, buildRetryPrompt, parseVerdict, runCoderTask } from '../doctor';
import type { DoctorVerdict } from '../doctor';
import type { ActionFailure, ActionRunResult } from '../types';

export type DoctorStatus =
  'idle' | 'diagnosing' | 'awaiting-review' | 'verifying' | 'fixed' | 'gave-up';

export interface DoctorRun {
  status: DoctorStatus;
  threadId?: string;
  attempts: number;
  verdict?: DoctorVerdict;
  message?: string;
}

const MAX_ATTEMPTS = 2;

const runs = $state<Record<string, DoctorRun>>({});
// Plain (non-reactive) per-session counter keyed by `${projectId}:${actionId}`.
const attemptCounts = new Map<string, number>();

export function getDoctorRun(projectId: string | number): DoctorRun {
  return runs[String(projectId)] ?? { status: 'idle', attempts: 0 };
}

/** Clear the visible doctor status for a project (e.g. when a fresh run starts),
 *  without resetting the per-session attempt bound. */
export function resetDoctorRun(projectId: string | number): void {
  runs[String(projectId)] = { status: 'idle', attempts: 0 };
}

/**
 * Diagnose and fix a failed action, re-running to confirm. Returns the final
 * verify run (so the UI can refresh its last-result and the failure card clears
 * on success), or null when no verify happened (environment/limit/flaky/error).
 */
export async function diagnose(
  project: Project,
  failure: ActionFailure
): Promise<ActionRunResult | null> {
  const pid = String(project.id);
  const key = `${pid}:${failure.actionId}`;
  const set = (patch: Partial<DoctorRun>) => {
    runs[pid] = { ...getDoctorRun(pid), ...patch };
  };

  if (failure.kind === 'environment') {
    set({
      status: 'gave-up',
      message: failure.hint ?? 'Environment issue — no code fix will help.',
    });
    return null;
  }
  if ((attemptCounts.get(key) ?? 0) >= MAX_ATTEMPTS) {
    set({ status: 'gave-up', message: 'Reached the fix-attempt limit.' });
    return null;
  }

  set({ status: 'diagnosing', message: undefined, verdict: undefined });
  let lastVerify: ActionRunResult | null = null;
  // Run the fix on whatever coder backend + model the user has configured (e.g. a
  // local Ollama model, free) — not a forced paid engine.
  const provider = coderSession.selectedBackendProvider;
  const model = coderSession.selectedModel;
  try {
    let prompt = buildDiagnosePrompt(failure);
    let current = failure;
    while ((attemptCounts.get(key) ?? 0) < MAX_ATTEMPTS) {
      const attempts = (attemptCounts.get(key) ?? 0) + 1;
      attemptCounts.set(key, attempts);
      set({ status: 'diagnosing', attempts });
      const r = await runCoderTask(project, prompt, provider, model);
      set({ threadId: r.threadId });
      if (r.cancelled) {
        set({ status: 'gave-up', message: 'Cancelled.' });
        return lastVerify;
      }
      const verdict = parseVerdict(r.finalText);
      set({ verdict });
      if (verdict === 'flaky' || verdict === 'environment') {
        set({
          status: 'gave-up',
          message:
            verdict === 'flaky'
              ? 'Looks flaky — no code change made.'
              : 'Environment issue — no code fix will help.',
        });
        return lastVerify;
      }
      set({ status: 'verifying' });
      const result = await actions.forProject(project).run(failure.actionId);
      lastVerify = result;
      if (result.success) {
        set({ status: 'fixed', message: 'Action passes now.' });
        return result;
      }
      current = result.failure ?? current;
      prompt = buildRetryPrompt(current);
    }
    set({ status: 'gave-up', message: 'Still failing after the fix attempts.' });
    return lastVerify;
  } catch (e) {
    set({ status: 'gave-up', message: e instanceof Error ? e.message : String(e) });
    return lastVerify;
  }
}
