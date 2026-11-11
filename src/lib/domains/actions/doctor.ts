import { coderService } from '$lib/domains/coder/services/coderService';
import type { Project } from '$lib/domains/projects/types';
import type { ActionFailure } from './types';

export type DoctorVerdict = 'app' | 'pipeline' | 'environment' | 'flaky' | 'unknown';

const VERDICT_LINE =
  'Your FINAL line must be exactly `DOCTOR_VERDICT: <app|pipeline|environment|flaky>` ' +
  '(pick one: app = you fixed app code, pipeline = you fixed the action command, ' +
  'environment = a machine/tooling problem code cannot fix, flaky = passes on retry).';

export function buildDiagnosePrompt(failure: ActionFailure): string {
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
  model?: string | null
): Promise<{ threadId: string; finalText: string | null; exhausted: boolean; cancelled: boolean }> {
  const thread = await coderService.createThread(
    project.path,
    model ?? undefined,
    provider ?? undefined,
    null,
    Number(project.id)
  );
  const threadId = thread.id;
  let finish!: (r: Awaited<ReturnType<typeof runCoderTask>>) => void;
  let fail!: (e: Error) => void;
  const done = new Promise<Awaited<ReturnType<typeof runCoderTask>>>((res, rej) => {
    finish = res;
    fail = rej;
  });
  // Subscribe (and wait for registration) before send so a fast completion is not missed.
  const unlisten = await Promise.all([
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
  ]);
  try {
    await coderService.send(threadId, prompt);
    return await done;
  } finally {
    unlisten.forEach((u) => u());
  }
}
