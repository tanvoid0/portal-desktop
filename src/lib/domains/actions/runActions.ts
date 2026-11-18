/**
 * What you can do to one action run (a `script_executions` row) — shared by the
 * run detail and the runs list's right-click menu so both behave the same.
 */
import {
  scriptExecutionService,
  type ScriptExecutionInfo,
} from '$lib/domains/scripts/services/scriptExecutionService';
import { invokeClient } from '$lib/utils/invokeClient';
import { confirmAction } from '$lib/utils/confirm';
import { toast } from '$lib/utils/toast';
import { runRequester, toTerminalText } from './runOutput';

const message = (err: unknown) => (err instanceof Error ? err.message : String(err));

export const isActive = (run: { status: string }) =>
  run.status === 'running' || run.status === 'pending' || run.status === 'queued';

/** Stop the run and its process tree. A run an MCP client asked for confirms
 *  first: that client reads the run back as stopped, not failed. */
export async function stopRun(exec: {
  id: string;
  triggeredBy: string;
  kind?: 'action' | 'pipeline' | 'release';
}): Promise<boolean> {
  const by = runRequester(exec.triggeredBy);
  if (
    by &&
    !(await confirmAction(
      `${by} started this run. Stopping it reports the run to that client as stopped by you.`,
      'Stop run?',
      { confirmLabel: 'Stop', destructive: true }
    ))
  ) {
    return false;
  }
  try {
    if (exec.kind && exec.kind !== 'action') {
      await invokeClient.post('cancel_pipeline_execution', { executionId: exec.id });
    } else {
      await scriptExecutionService.cancelExecution(exec.id);
    }
    toast.info('Run stopped');
    return true;
  } catch (err) {
    toast.error('Could not stop run', message(err));
    return false;
  }
}

/** Start the same command in the same folder. Returns the new run's id. */
export async function rerun(exec: ScriptExecutionInfo): Promise<string | null> {
  try {
    return await scriptExecutionService.executeScript({
      command: exec.command,
      workingDirectory: exec.workingDirectory ?? undefined,
      // Same as an Actions run: non-interactive (QA-129).
      env: { CI: 'true' },
    });
  } catch (err) {
    toast.error('Could not start run', message(err));
    return null;
  }
}

export async function deleteRun(exec: { id: string }): Promise<boolean> {
  const ok = await confirmAction(
    'Delete this run and its output? This cannot be undone.',
    'Delete run?',
    { confirmLabel: 'Delete', destructive: true }
  );
  if (!ok) return false;
  try {
    await scriptExecutionService.deleteExecution(exec.id);
    return true;
  } catch (err) {
    toast.error('Could not delete run', message(err));
    return false;
  }
}

export async function copyRunText(kind: 'command' | 'output', exec: ScriptExecutionInfo) {
  try {
    const text =
      kind === 'command'
        ? exec.command
        : toTerminalText(
            isActive(exec)
              ? (await scriptExecutionService.getLiveOutput(exec.id)).join('')
              : exec.output
          );
    await navigator.clipboard.writeText(text);
    toast.success(kind === 'command' ? 'Command copied' : 'Output copied');
  } catch {
    toast.error('Could not copy');
  }
}
