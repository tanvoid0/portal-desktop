import { describe, expect, it } from 'vitest';
import { stepRecordPayload } from './stepRecord';

const run = { workflowRunId: 'r1', workflowId: 'w1', workflowPlan: [] };

describe('stepRecordPayload', () => {
  it('links a failed github step to the run and carries its error', () => {
    const p = stepRecordPayload(run, {
      name: 'Deploy',
      actionId: 'gh-deploy',
      status: 'failed',
      error: 'boom',
      runner: 'github',
    });
    expect(p).toMatchObject({
      command: 'Deploy',
      exitCode: null,
      status: 'failed',
      error: 'boom',
      workflowRunId: 'r1',
      stepId: 'gh-deploy',
    });
  });

  it('records success with exit code 0', () => {
    const p = stepRecordPayload(run, {
      name: 'x',
      actionId: 'x',
      status: 'success',
      runner: 'n8n',
    });
    expect(p.exitCode).toBe(0);
  });
});
