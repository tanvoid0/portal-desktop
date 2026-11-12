import { coderService } from '$lib/domains/coder/services/coderService';
import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
import type { Project } from '$lib/domains/projects/types';
import type {
  ActionFailure,
  ProjectAutomationProfile,
  UnifiedAction,
  UnifiedWorkflow,
} from './types';

export type DoctorVerdict = 'app' | 'pipeline' | 'environment' | 'flaky' | 'unknown';

/** `.portal/pipeline.yml` schema (version 1), shown to the agent verbatim. */
export const PIPELINE_FILE_SCHEMA = [
  'version: 1',
  'actions:',
  '  <action-id>:',
  '    name: <optional label>',
  '    run: <shell command>        # required, non-empty',
  '    cwd: <optional, relative to project root>',
  '    category: <optional>',
  '    longRunning: <optional true|false>',
  'workflows:',
  '  <workflow-id>:',
  '    steps:',
  '      - action: <action-id>',
  '        needs: [<action-id>, ...]',
].join('\n');

const VERDICT_LINE =
  'Your FINAL line must be exactly `DOCTOR_VERDICT: <app|pipeline|environment|flaky>` ' +
  '(pick one: app = you fixed app code, pipeline = you fixed the action command, ' +
  'environment = a machine/tooling problem code cannot fix, flaky = passes on retry).';

/**
 * Guidance appended when the failure looks like a pipeline problem (wrong/missing
 * command, not app code): the `.portal/pipeline.yml` schema + the current action, so
 * the agent overrides the action by id instead of guessing the file format. A file
 * action with the same id beats Portal's built-in default (resolver `mergeCatalog`).
 */
function pipelineFixGuidance(failure: ActionFailure, action?: UnifiedAction): string {
  return [
    '',
    'This looks like a PIPELINE problem: the action command itself is wrong or missing,',
    "not the app's code. Fix it by adding or overriding this action in the project's",
    '`.portal/pipeline.yml` (create it if absent). A file action with the SAME id overrides',
    "Portal's built-in default — keep the id unchanged.",
    '',
    `Failing action id: ${failure.actionId}`,
    `Current (wrong) command: ${action?.command ?? failure.command ?? '(unknown)'}`,
    '',
    '`.portal/pipeline.yml` schema (version 1):',
    '```yaml',
    PIPELINE_FILE_SCHEMA,
    '```',
  ].join('\n');
}

export function buildDiagnosePrompt(failure: ActionFailure, action?: UnifiedAction): string {
  return [
    'A Portal Action command failed. Diagnose the root cause and fix it, then stop.',
    '',
    `Command: ${failure.command ?? '(unknown)'}`,
    `Exit code: ${failure.exitCode ?? 'unknown'}`,
    `Working directory: ${failure.cwd}`,
    `Failure kind: ${failure.kind}`,
    '',
    'Output tail:',
    '```',
    failure.outputTail,
    '```',
    '',
    "Fix either the app's code, or the action's command definition in the project's " +
      '.portal/pipeline.yml if the command itself is wrong or missing.',
    ...(failure.kind === 'pipeline' ? [pipelineFixGuidance(failure, action)] : []),
    '',
    'Do NOT run git commit or git push.',
    VERDICT_LINE,
  ].join('\n');
}

export function buildRetryPrompt(failure: ActionFailure): string {
  return [
    'The action still fails after your change. New output tail:',
    '```',
    failure.outputTail,
    '```',
    'Try again, then stop. Do NOT run git commit or git push.',
    VERDICT_LINE,
  ].join('\n');
}

export function parseVerdict(finalText: string | null): DoctorVerdict {
  const all = [
    ...(finalText ?? '').matchAll(/DOCTOR_VERDICT:\s*(app|pipeline|environment|flaky)/gi),
  ];
  const last = all.at(-1);
  return last ? (last[1].toLowerCase() as DoctorVerdict) : 'unknown';
}

/**
 * Stage 4 — generate/customise the project's pipeline from a natural-language
 * requirement. The agent writes or extends `.portal/pipeline.yml` (plus any
 * scripts it references) so the action catalog satisfies the requirement, given
 * the detected stack (`profile`) and what already exists (`catalog`). Ends with a
 * `DOCTOR_GENERATED: <id>` line naming the primary action/workflow it added, so
 * the caller can dry-check it with `actions.preview`.
 */
export function buildGeneratePrompt(
  requirement: string,
  profile: ProjectAutomationProfile,
  catalog: { actions: UnifiedAction[]; workflows: UnifiedWorkflow[] },
  schema: string
): string {
  const frameworks = profile.frameworks.length ? profile.frameworks.join(', ') : 'unknown';
  const actionList = catalog.actions.length
    ? catalog.actions.map((a) => `  - ${a.id}${a.command ? `: ${a.command}` : ''}`).join('\n')
    : '  (none)';
  const workflowList = catalog.workflows.length
    ? catalog.workflows
        .map((w) => `  - ${w.id}: ${w.steps.map((s) => s.action).join(' → ') || '(no steps)'}`)
        .join('\n')
    : '  (none)';
  return [
    "Create or extend this project's Portal pipeline to satisfy a requirement, then stop.",
    '',
    `Requirement: ${requirement}`,
    '',
    'Detected stack:',
    `- Package manager: ${profile.packageManager}`,
    `- Framework(s): ${frameworks}`,
    `- Project root: ${profile.path}`,
    '',
    'Current actions:',
    actionList,
    'Current workflows:',
    workflowList,
    '',
    'Write or extend `.portal/pipeline.yml` (create it if absent) so the catalog satisfies the',
    'requirement, and add any scripts the actions reference. Use real commands for the',
    `${profile.packageManager} package manager and the detected framework — no placeholders. Reuse`,
    "existing action ids where it fits; a file action with the SAME id overrides Portal's built-in",
    'default. Keep every `run` command non-empty and make workflow steps reference known actions.',
    '',
    '`.portal/pipeline.yml` schema (version 1):',
    '```yaml',
    schema,
    '```',
    '',
    'Do NOT run git commit or git push.',
    'Your FINAL line must be exactly `DOCTOR_GENERATED: <id>`, naming the primary action or',
    'workflow id you added or changed (so Portal can dry-check it).',
  ].join('\n');
}

/** Pull the `DOCTOR_GENERATED: <id>` the generate agent emits; null if absent. */
export function parseGenerated(finalText: string | null): string | null {
  const all = [...(finalText ?? '').matchAll(/DOCTOR_GENERATED:\s*([A-Za-z0-9._-]+)/gi)];
  return all.at(-1)?.[1] ?? null;
}

/** The coder backend cannot start a run (Agent Platform down). Not a failed fix attempt. */
export class CoderUnavailableError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CoderUnavailableError';
  }
}

/** Live hooks for a coder run: the thread exists / it is blocked on the owner. */
export interface CoderTaskHooks {
  /** Reuse this thread (a retry) instead of creating one. */
  threadId?: string;
  /** Thread created; `null` when it was deleted again because the run never started. */
  onThread?: (threadId: string | null) => void;
  onWaiting?: (waiting: boolean) => void;
}

export async function runCoderTask(
  project: Project,
  prompt: string,
  /**
   * Which coder backend + model to run the fix on. Defaults to the user's
   * configured provider/model (passed in by the caller) — which may be a local,
   * free model (e.g. Ollama) or a provider they already pay for. `null`/undefined
   * lets the backend pick its default. We no longer force `claude-cli`.
   */
  provider?: string | null,
  model?: string | null,
  hooks: CoderTaskHooks = {}
): Promise<{ threadId: string; finalText: string | null; exhausted: boolean; cancelled: boolean }> {
  type Result = Awaited<ReturnType<typeof runCoderTask>>;
  const created = !hooks.threadId;
  let threadId = hooks.threadId ?? '';
  const unlisten: Array<() => void> = [];
  let finish!: (r: Result) => void;
  let fail!: (e: Error) => void;
  const done = new Promise<Result>((res, rej) => {
    finish = res;
    fail = rej;
  });
  try {
    // Anything that goes wrong before the run starts is "could not start", not a
    // failed fix: no agent ran, no attempt is spent, no empty thread is left.
    try {
      // Every backend except the claude CLI goes through the Agent Platform.
      if (provider !== 'claude-cli') {
        await aiPlatform.refresh();
        // refresh() returns early when a check is already in flight: wait for it.
        while (aiPlatform.checking) await new Promise((r) => setTimeout(r, 50));
        if (aiPlatform.blocked) {
          throw new Error(
            aiPlatform.status === 'unreachable'
              ? 'Agent Platform is not running.'
              : aiPlatform.status === 'not_installed'
                ? 'Agent Platform is not installed.'
                : 'Agent Platform rejected this app’s token.'
          );
        }
      }
      if (created) {
        const thread = await coderService.createThread(
          project.path,
          model ?? undefined,
          provider ?? undefined,
          null,
          Number(project.id)
        );
        threadId = thread.id;
        hooks.onThread?.(threadId);
      }
      // Subscribe (and wait for registration) before send so a fast completion is not missed.
      const subs = await Promise.allSettled([
        coderService.onDone((p) => {
          if (p.thread_id === threadId)
            finish({
              threadId,
              finalText: p.final_text,
              exhausted: p.exhausted,
              cancelled: !!p.cancelled,
            });
        }),
        coderService.onError((p) => {
          if (p.thread_id === threadId) fail(new Error(p.error));
        }),
        coderService.onPending((p) => {
          if (p.thread_id === threadId) hooks.onWaiting?.(true);
        }),
        coderService.onMessage((p) => {
          if (p.thread_id === threadId) hooks.onWaiting?.(false);
        }),
      ]);
      for (const s of subs) if (s.status === 'fulfilled') unlisten.push(s.value);
      const rejected = subs.find((s) => s.status === 'rejected');
      if (rejected) throw (rejected as PromiseRejectedResult).reason;
      await coderService.send(threadId, prompt);
    } catch (e) {
      if (created && threadId) {
        await coderService.deleteThread(threadId).catch(() => {});
        hooks.onThread?.(null);
      }
      throw new CoderUnavailableError(e instanceof Error ? e.message : String(e));
    }
    return await done;
  } finally {
    unlisten.forEach((u) => u());
  }
}
