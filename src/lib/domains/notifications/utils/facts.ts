import type { PipelineExecution } from '$lib/domains/projects/pipelines';
import { formatDuration } from '$lib/domains/terminal/utils/durationUtils';
import type { NotificationContext } from '../state/notificationInbox.svelte';

type Facts = NonNullable<NotificationContext['facts']>;

// History lives in localStorage: a snippet is enough, a whole heredoc is not.
const MAX_VALUE = 200;

/** What an approval is about: the tool, plus the command or file it touches. */
export function permissionFact(toolName: string, input: unknown): Facts {
  const facts: Facts = [{ kind: 'tool', value: toolName }];
  const args = (input ?? {}) as Record<string, unknown>;
  if (typeof args.command === 'string') {
    facts.push({ kind: 'command', value: args.command.slice(0, MAX_VALUE) });
  } else if (typeof args.file_path === 'string') {
    facts.push({ kind: 'file', value: args.file_path.slice(0, MAX_VALUE) });
  }
  return facts;
}

/** Where a finished pipeline broke and how long it took. */
export function pipelineFacts(execution: PipelineExecution): Facts {
  const facts: Facts = [];
  const failed = execution.stepExecutions?.find((s) => s.status === 'failed');
  if (failed) {
    facts.push({ kind: 'step', value: failed.stepName });
    if (failed.exitCode != null) facts.push({ kind: 'exit', value: String(failed.exitCode) });
  }
  // Dates arrive as strings over the event, whatever the type says.
  const ms =
    new Date(execution.finishedAt ?? NaN).getTime() - new Date(execution.startedAt).getTime();
  if (ms >= 0) facts.push({ kind: 'duration', value: formatDuration(ms) });
  return facts;
}
