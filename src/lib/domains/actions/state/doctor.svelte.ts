import type { Project } from '$lib/domains/projects/types';
import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
import { actions } from '../actionsService';
import {
  PIPELINE_FILE_SCHEMA,
  buildDiagnosePrompt,
  buildGeneratePrompt,
  buildRetryPrompt,
  parseGenerated,
  parseVerdict,
  CoderUnavailableError,
  runCoderTask,
} from '../doctor';
import type { DoctorVerdict } from '../doctor';
import { reloadAndValidatePortalPipelineFile } from '../pipelineFile';
import { resolveProjectCatalog } from '../resolver';
import type { ActionCatalog } from '../resolver';
import { profileFromProject } from '../profile';
import type { ActionFailure, ActionRunResult, UnifiedAction } from '../types';

export type DoctorStatus =
  'idle' | 'diagnosing' | 'awaiting-verify' | 'verifying' | 'fixed' | 'gave-up';

/**
 * The agent changed what the action would run. It is shown to the owner and only
 * run when they click Verify — an agent-written command never executes unseen (QA-153).
 */
export interface VerifyProposal {
  actionId: string;
  oldCommand: string;
  newCommand: string;
  /** package.json scripts the agent added or changed ("name: body"). */
  scriptChanges: string[];
  /** Why confirmation is needed when it is not just a changed command. */
  note?: string;
}

/** Everything that decides what `run([actionId])` would execute. */
interface Snapshot {
  exists: boolean;
  command: string;
  scripts: Record<string, string>;
  /** A workflow shares the action's id (workflows win a by-id run). */
  shadowed: boolean;
}

function snapshot(
  catalog: Pick<ActionCatalog, 'actions' | 'workflows' | 'profile'>,
  id: string
): Snapshot {
  const a = catalog.actions.find((x) => x.id === id);
  return {
    exists: !!a,
    command: a?.command ?? '',
    scripts: catalog.profile?.scripts ?? {},
    shadowed: catalog.workflows.some((w) => w.id === id),
  };
}

const sameSnapshot = (a: Snapshot, b: Snapshot) =>
  a.command.trim() === b.command.trim() && JSON.stringify(a.scripts) === JSON.stringify(b.scripts);

/** Fail closed: no baseline, a missing/shadowed action or any change needs the owner. */
export function needsConfirm(before: Snapshot | null, after: Snapshot): boolean {
  return !before || !after.exists || after.shadowed || !sameSnapshot(before, after);
}

function proposalOf(actionId: string, before: Snapshot | null, after: Snapshot): VerifyProposal {
  const scriptChanges = Object.entries(after.scripts)
    .filter(([k, v]) => before?.scripts[k] !== v)
    .map(([k, v]) => `${k}: ${v}`);
  return {
    actionId,
    oldCommand: before?.command ?? '',
    newCommand: after.command,
    scriptChanges,
    note: !after.exists
      ? 'The action no longer exists.'
      : after.shadowed
        ? 'A workflow now shares this id.'
        : undefined,
  };
}

/**
 * Surfaced (never auto-applied) when a Portal *default* command was wrong and a
 * `.portal/pipeline.yml` override fixed it — the facts needed to improve Portal's
 * own `defaults.ts` generation. Portal's source is not the agent's workspace, so
 * the doctor reports this instead of editing it (Stage 3).
 */
export interface PipelineFixReport {
  actionId: string;
  packageManager: string;
  frameworks: string[];
  /** Portal's built-in default command that failed. */
  defaultCommand: string;
  /** The command that passed after the override. */
  greenCommand: string;
}

export interface DoctorRun {
  status: DoctorStatus;
  threadId?: string;
  attempts: number;
  verdict?: DoctorVerdict;
  message?: string;
  report?: PipelineFixReport;
  /** The agent is blocked on an approval in Coder. */
  waiting?: boolean;
  /** Agent platform / backend could not start a run (not a used attempt). */
  unavailable?: boolean;
  proposal?: VerifyProposal;
  /**
   * The failure being worked on, cleared once fixed. Lives here, not in the panel,
   * so the failure/status card survives leaving the page mid-run (QA-156).
   */
  failure?: ActionFailure;
  /**
   * Coder thread whose file changes are still pending review. Survives
   * `resetDoctorRun` so the pointer outlives the failure card (QA-152).
   */
  reviewThreadId?: string;
}

interface PendingVerify {
  failure: ActionFailure;
  verdict: DoctorVerdict;
  originalAction?: UnifiedAction;
  before: Snapshot | null;
  /** Exactly what the owner was shown; Verify runs only if the disk still matches. */
  shown: Snapshot;
}

const MAX_ATTEMPTS = 2;

const runs = $state<Record<string, DoctorRun>>({});
// Plain (non-reactive) per-session counter keyed by `${projectId}:${actionId}`.
const attemptCounts = new Map<string, number>();
// Fix awaiting the owner's Verify click, keyed by project id.
const pendingVerify = new Map<string, PendingVerify>();

export function getDoctorRun(projectId: string | number): DoctorRun {
  return runs[String(projectId)] ?? { status: 'idle', attempts: 0 };
}

/** Clear the visible doctor status for a project (e.g. when a fresh run starts),
 *  without resetting the per-session attempt bound. */
export function resetDoctorRun(projectId: string | number): void {
  const id = String(projectId);
  pendingVerify.delete(id);
  runs[id] = { status: 'idle', attempts: 0, reviewThreadId: runs[id]?.reviewThreadId };
}

/** Drop the "changes pending review" pointer (owner dismissed it). */
export function dismissReviewPointer(projectId: string | number): void {
  const id = String(projectId);
  if (runs[id]) runs[id] = { ...runs[id], reviewThreadId: undefined };
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
  set({ failure });

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

  // Resolve the failing action from the local catalog *before* any fix, so we can
  // (a) show the agent the current command and (b) tell a Portal default
  // (source 'local') from an existing file override — only a default earns the
  // report card after a pipeline fix.
  let originalAction: UnifiedAction | undefined;
  let before: Snapshot | null = null;
  try {
    const catalog = await resolveProjectCatalog(project);
    originalAction = catalog.actions.find((a) => a.id === failure.actionId);
    before = snapshot(catalog, failure.actionId);
  } catch {
    /* best-effort; prompt + report degrade gracefully without it */
  }

  pendingVerify.delete(pid);
  set({
    status: 'diagnosing',
    message: undefined,
    verdict: undefined,
    proposal: undefined,
    report: undefined,
    waiting: false,
    unavailable: false,
  });
  let lastVerify: ActionRunResult | null = null;
  let threadId: string | undefined; // reused across retries
  // Run the fix on whatever coder backend + model the user has configured (e.g. a
  // local Ollama model, free) — not a forced paid engine.
  const provider = coderSession.selectedBackendProvider;
  const model = coderSession.selectedModel;
  try {
    let prompt = buildDiagnosePrompt(failure, originalAction);
    let current = failure;
    while ((attemptCounts.get(key) ?? 0) < MAX_ATTEMPTS) {
      const attempts = (attemptCounts.get(key) ?? 0) + 1;
      attemptCounts.set(key, attempts);
      set({ status: 'diagnosing', attempts });
      const r = await runCoderTask(project, prompt, provider, model, {
        threadId,
        // Link the thread the moment it exists, not after the run (QA-151).
        onThread: (id) => set({ threadId: id ?? undefined, reviewThreadId: id ?? undefined }),
        onWaiting: (waiting) => set({ waiting }),
      });
      threadId = r.threadId;
      set({ threadId, reviewThreadId: threadId, waiting: false });
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
      // A pipeline fix edits `.portal/pipeline.yml`. Reload + validate it first so an
      // invalid edit is surfaced, not silently ignored (loadPortalPipelineFile would
      // swallow a parse error and fall back to the wrong default, looping forever).
      if (verdict === 'pipeline') {
        try {
          await reloadAndValidatePortalPipelineFile(project.path);
        } catch (e) {
          set({
            status: 'gave-up',
            message: `The agent wrote an invalid .portal/pipeline.yml — ${
              e instanceof Error ? e.message : String(e)
            }`,
          });
          return lastVerify;
        }
      }
      // Whatever the agent did to the command/scripts/workflows, the owner sees it
      // before it runs. Fail closed: an unreadable catalog never auto-verifies.
      let after: Snapshot;
      try {
        after = snapshot(await resolveProjectCatalog(project), failure.actionId);
      } catch (e) {
        set({
          status: 'gave-up',
          message: `Could not re-read the actions to check the agent's change (${
            e instanceof Error ? e.message : String(e)
          }). Review it in Coder, then run the action yourself.`,
        });
        return lastVerify;
      }
      if (needsConfirm(before, after)) {
        propose(pid, set, { failure: current, verdict, originalAction, before, shown: after });
        return lastVerify;
      }
      const ran = await verifyRun(project, failure.actionId, verdict, originalAction, after, set);
      if (ran.kind === 'changed') {
        propose(pid, set, { failure: current, verdict, originalAction, before, shown: ran.now });
        return lastVerify;
      }
      const result = ran.result;
      lastVerify = result;
      if (result.success) {
        attemptCounts.delete(key);
        return result;
      }
      current = result.failure ?? current;
      prompt = buildRetryPrompt(current);
    }
    set({ status: 'gave-up', message: 'Still failing after the fix attempts.' });
    return lastVerify;
  } catch (e) {
    if (e instanceof CoderUnavailableError) {
      // No agent ever ran: not a used attempt, and not "gave up".
      attemptCounts.set(key, Math.max(0, (attemptCounts.get(key) ?? 1) - 1));
      set({ status: 'gave-up', unavailable: true, waiting: false, message: e.message });
    } else {
      set({
        status: 'gave-up',
        waiting: false,
        message: e instanceof Error ? e.message : String(e),
      });
    }
    return lastVerify;
  }
}

function propose(
  pid: string,
  set: (patch: Partial<DoctorRun>) => void,
  pending: PendingVerify
): void {
  pendingVerify.set(pid, pending);
  set({
    status: 'awaiting-verify',
    message: 'The agent changed what this action runs. Review it, then Verify.',
    proposal: proposalOf(pending.failure.actionId, pending.before, pending.shown),
  });
}

/**
 * Run the action (the fix's verify step) — the action only, never a workflow that
 * shares its id — after re-checking that disk still matches `expected`. Runs the
 * exact resolved action that was checked (no re-resolve), so a `.portal/pipeline.yml`
 * write after the check cannot change what executes.
 * ponytail: package.json scripts are read by the package manager at spawn, so a
 * scripts write in that sub-second gap is not caught; closing it means inlining
 * the script body into the command.
 */
async function verifyRun(
  project: Project,
  actionId: string,
  verdict: DoctorVerdict,
  originalAction: UnifiedAction | undefined,
  expected: Snapshot,
  set: (patch: Partial<DoctorRun>) => void
): Promise<{ kind: 'ran'; result: ActionRunResult } | { kind: 'changed'; now: Snapshot }> {
  const catalog = await resolveProjectCatalog(project);
  const now = snapshot(catalog, actionId);
  const action = catalog.actions.find((a) => a.id === actionId);
  if (!action || now.shadowed || !sameSnapshot(expected, now)) return { kind: 'changed', now };
  set({ status: 'verifying', proposal: undefined });
  const result = await actions.forProject(project).runResolved([action]);
  if (result.success) {
    const report =
      verdict === 'pipeline' && originalAction?.source === 'local'
        ? pipelineFixReport(project, actionId, originalAction, result)
        : undefined;
    set({ status: 'fixed', message: 'Action passes now.', report, failure: undefined });
  } else if (result.failure) {
    set({ failure: result.failure });
  }
  return { kind: 'ran', result };
}

/**
 * The owner clicked Verify: run exactly what they were shown. If disk changed
 * since, show the new version instead of running it. Returns the verify run, or
 * null when nothing ran. A red run is left to the failure card; no auto-retry.
 */
export async function verifyProposal(project: Project): Promise<ActionRunResult | null> {
  const pid = String(project.id);
  const pending = pendingVerify.get(pid);
  if (!pending) return null;
  pendingVerify.delete(pid);
  const set = (patch: Partial<DoctorRun>) => {
    runs[pid] = { ...getDoctorRun(pid), ...patch };
  };
  try {
    const ran = await verifyRun(
      project,
      pending.failure.actionId,
      pending.verdict,
      pending.originalAction,
      pending.shown,
      set
    );
    if (ran.kind === 'changed') {
      propose(pid, set, { ...pending, shown: ran.now });
      return null;
    }
    if (ran.result.success) attemptCounts.delete(`${pid}:${pending.failure.actionId}`);
    else set({ status: 'gave-up', message: 'Still failing with the new command.' });
    return ran.result;
  } catch (e) {
    set({ status: 'gave-up', message: e instanceof Error ? e.message : String(e) });
    return null;
  }
}
// ─── Stage 4: requirement-driven generation ──────────────────────────────────

export type GenerateStatus =
  'idle' | 'generating' | 'validating' | 'previewing' | 'done' | 'failed';

export interface GenerateRun {
  status: GenerateStatus;
  threadId?: string;
  message?: string;
  /** The action/workflow id the agent said it added — the dry-check target. */
  targetId?: string;
  /** Whether the dry-check (preview) compiled to a runnable plan. */
  previewReady?: boolean;
  waiting?: boolean;
  unavailable?: boolean;
}

const genRuns = $state<Record<string, GenerateRun>>({});

export function getGenerateRun(projectId: string | number): GenerateRun {
  return genRuns[String(projectId)] ?? { status: 'idle' };
}

/**
 * Generate/customise `.portal/pipeline.yml` from a natural-language requirement on
 * the user's configured coder backend, then dry-check it: reload+validate the file
 * (surface an invalid edit), and `preview` the action/workflow the agent named
 * (compile the plan, never run it). The panel reloads its catalog after. Returns
 * the coder thread id (for the "Review in Coder" link) or null when none spawned.
 */
export async function generatePipeline(
  project: Project,
  requirement: string
): Promise<string | null> {
  const pid = String(project.id);
  const set = (patch: Partial<GenerateRun>) => {
    genRuns[pid] = { ...getGenerateRun(pid), ...patch };
  };
  const req = requirement.trim();
  if (!req) {
    set({ status: 'failed', message: 'Describe the pipeline you want first.' });
    return null;
  }

  set({
    status: 'generating',
    message: undefined,
    threadId: undefined,
    targetId: undefined,
    previewReady: undefined,
    waiting: false,
    unavailable: false,
  });
  // Run on whatever coder backend + model the user has configured (local = free).
  const provider = coderSession.selectedBackendProvider;
  const model = coderSession.selectedModel;
  try {
    const catalog = await resolveProjectCatalog(project);
    const profile = profileFromProject(project);
    const prompt = buildGeneratePrompt(req, profile, catalog, PIPELINE_FILE_SCHEMA);
    const r = await runCoderTask(project, prompt, provider, model, {
      onThread: (id) => set({ threadId: id ?? undefined }),
      onWaiting: (waiting) => set({ waiting }),
    });
    set({ threadId: r.threadId, waiting: false });
    if (r.cancelled) {
      set({ status: 'failed', message: 'Cancelled.' });
      return r.threadId;
    }

    // Surface an invalid agent edit instead of silently falling back to a default.
    set({ status: 'validating' });
    try {
      await reloadAndValidatePortalPipelineFile(project.path);
    } catch (e) {
      set({
        status: 'failed',
        message: `The agent wrote an invalid .portal/pipeline.yml — ${
          e instanceof Error ? e.message : String(e)
        }`,
      });
      return r.threadId;
    }

    // Dry-check the action/workflow the agent named: compile its plan, don't run.
    const targetId = parseGenerated(r.finalText);
    set({ targetId: targetId ?? undefined });
    if (targetId) {
      set({ status: 'previewing' });
      const preview = await actions.forProject(project).preview(targetId);
      if (!preview.ready) {
        set({
          status: 'failed',
          previewReady: false,
          message:
            preview.errors[0] ??
            `Generated "${targetId}" is not a runnable plan — review it in Coder.`,
        });
        return r.threadId;
      }
      set({ previewReady: true });
    }

    set({
      status: 'done',
      message: targetId
        ? `Added "${targetId}". Review the changes in Coder before running.`
        : 'Pipeline written. Review the changes in Coder.',
    });
    return r.threadId;
  } catch (e) {
    set({
      status: 'failed',
      waiting: false,
      unavailable: e instanceof CoderUnavailableError,
      message: e instanceof Error ? e.message : String(e),
    });
    return getGenerateRun(pid).threadId ?? null;
  }
}

function pipelineFixReport(
  project: Project,
  actionId: string,
  original: UnifiedAction,
  verify: ActionRunResult
): PipelineFixReport {
  const profile = profileFromProject(project);
  const greenCommand =
    verify.steps.find((s) => s.actionId === actionId)?.command ?? original.command ?? '';
  return {
    actionId,
    packageManager: profile.packageManager,
    frameworks: profile.frameworks,
    defaultCommand: original.command ?? '',
    greenCommand,
  };
}
