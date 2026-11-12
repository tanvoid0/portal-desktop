import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Project } from '$lib/domains/projects/types';
import type { ActionFailure } from './types';

const h = vi.hoisted(() => ({
  platform: { blocked: false, status: 'ok' as string, refresh: async () => {} },
  createThread: vi.fn(),
  send: vi.fn(),
  deleteThread: vi.fn(async () => true),
  doneCb: null as null | ((p: unknown) => void),
  run: vi.fn(),
  runById: vi.fn(),
  afterResolve: null as null | (() => void),
  command: 'pnpm test',
  calls: 0,
  throwFrom: null as number | null,
  removed: false,
  scripts: {} as Record<string, string>,
  workflows: [] as { id: string }[],
}));

vi.mock('$lib/domains/ai/state/aiPlatform.svelte.js', () => ({ aiPlatform: h.platform }));
vi.mock('$lib/domains/coder/state/coderSession.svelte.js', () => ({
  coderSession: { selectedBackendProvider: null, selectedModel: null },
}));
vi.mock('$lib/domains/coder/services/coderService', () => ({
  coderService: {
    createThread: h.createThread,
    send: h.send,
    deleteThread: h.deleteThread,
    onDone: async (cb: (p: unknown) => void) => {
      h.doneCb = cb;
      return () => {};
    },
    onError: async () => () => {},
    onPending: async () => () => {},
    onMessage: async () => () => {},
  },
}));
// `run` (by id) re-resolves the catalog; the doctor must only use `runResolved`.
vi.mock('./actionsService', () => ({
  actions: { forProject: () => ({ run: h.runById, runResolved: h.run }) },
}));
vi.mock('./resolver', () => ({
  resolveProjectCatalog: async () => {
    const n = h.calls++;
    if (h.throwFrom !== null && n >= h.throwFrom) throw new Error('catalog unreadable');
    const write = h.afterResolve; // a disk write landing right after this read
    h.afterResolve = null;
    const out = {
      profile: { scripts: n === 0 ? {} : h.scripts },
      actions: [
        ...(h.removed && n > 0
          ? []
          : [
              {
                id: 'test',
                name: 'Test',
                source: 'local',
                runner: 'local',
                command: n === 0 ? 'pnpm test' : h.command,
              },
            ]),
      ],
      workflows: n > 0 ? h.workflows : [],
    };
    write?.();
    return out;
  },
}));
vi.mock('./pipelineFile', () => ({ reloadAndValidatePortalPipelineFile: async () => {} }));
vi.mock('./profile', () => ({
  profileFromProject: () => ({
    name: 'demo',
    path: 'D:/demo',
    packageManager: 'pnpm',
    frameworks: [],
  }),
}));

import { diagnose, getDoctorRun, verifyProposal } from './state/doctor.svelte';

let seq = 100;
let project = { id: 7, path: 'D:/demo', name: 'demo' } as unknown as Project;
const failure: ActionFailure = {
  actionId: 'test',
  runner: 'local',
  command: 'pnpm test',
  exitCode: 1,
  outputTail: 'Missing script: test',
  cwd: 'D:/demo',
  target: 'test',
  kind: 'pipeline',
};

beforeEach(() => {
  vi.clearAllMocks();
  project = { id: seq++, path: 'D:/demo', name: 'demo' } as unknown as Project; // fresh attempt counters
  h.platform.blocked = false;
  h.platform.status = 'ok';
  h.command = 'pnpm test';
  h.calls = 0;
  h.throwFrom = null;
  h.removed = false;
  h.scripts = {};
  h.workflows = [];
  h.afterResolve = null;
  h.createThread.mockResolvedValue({ id: 't1' });
  // The agent "finishes" as soon as the prompt is sent.
  h.send.mockImplementation(async () => {
    h.doneCb?.({
      thread_id: 't1',
      final_text: 'DOCTOR_VERDICT: pipeline',
      exhausted: false,
      cancelled: false,
    });
  });
  h.run.mockResolvedValue({ success: true, steps: [], cwd: 'D:/demo' });
});

describe('Agent Platform down (QA-150)', () => {
  it('gives a clear message, makes no thread and uses no attempt', async () => {
    h.platform.blocked = true;
    h.platform.status = 'unreachable';
    await diagnose(project, failure);
    const run = getDoctorRun(project.id);
    expect(run.unavailable).toBe(true);
    expect(run.message).toBe('Agent Platform is not running.');
    expect(h.createThread).not.toHaveBeenCalled();
    // Attempt not consumed: a second click still reaches the preflight, not "limit".
    await diagnose(project, failure);
    await diagnose(project, failure);
    expect(getDoctorRun(project.id).message).toBe('Agent Platform is not running.');
  });
});

describe('agent-written command needs a Verify click (QA-153)', () => {
  it('stops before running a changed command, runs it only on verifyProposal', async () => {
    h.command = 'pnpm typecheck'; // what the agent "wrote" into pipeline.yml
    const out = await diagnose(project, { ...failure });
    expect(out).toBeNull();
    expect(h.run).not.toHaveBeenCalled();
    const run = getDoctorRun(project.id);
    expect(run.status).toBe('awaiting-verify');
    expect(run.proposal?.newCommand).toBe('pnpm typecheck');
    expect(run.threadId).toBe('t1');
    expect(run.reviewThreadId).toBe('t1');

    const verify = await verifyProposal(project);
    expect(h.run).toHaveBeenCalledTimes(1);
    expect(verify?.success).toBe(true);
    expect(getDoctorRun(project.id).status).toBe('fixed');
  });
});

describe('fail closed (QA-153)', () => {
  it('a workflow shadowing the action id never auto-runs', async () => {
    h.workflows = [{ id: 'test' }]; // command string unchanged, but a workflow now owns the id
    const out = await diagnose(project, { ...failure });
    expect(out).toBeNull();
    expect(h.run).not.toHaveBeenCalled();
    expect(getDoctorRun(project.id).status).toBe('awaiting-verify');
    expect(getDoctorRun(project.id).proposal?.note).toContain('workflow');
  });

  it('a removed action never auto-runs', async () => {
    h.removed = true;
    await diagnose(project, { ...failure });
    expect(h.run).not.toHaveBeenCalled();
    expect(getDoctorRun(project.id).proposal?.note).toContain('no longer exists');
  });

  it('a changed package.json script needs Verify even if the command string is the same', async () => {
    h.scripts = { test: 'rm -rf /' };
    await diagnose(project, { ...failure });
    expect(h.run).not.toHaveBeenCalled();
    expect(getDoctorRun(project.id).proposal?.scriptChanges).toEqual(['test: rm -rf /']);
  });

  it('an unreadable catalog after the agent never runs anything', async () => {
    h.throwFrom = 1;
    await diagnose(project, { ...failure });
    expect(h.run).not.toHaveBeenCalled();
    expect(getDoctorRun(project.id).status).toBe('gave-up');
  });

  it('Verify re-shows instead of running when disk changed after the proposal', async () => {
    h.command = 'pnpm typecheck';
    await diagnose(project, { ...failure });
    h.command = 'rm -rf /'; // changed again before the click
    const out = await verifyProposal(project);
    expect(out).toBeNull();
    expect(h.run).not.toHaveBeenCalled();
    expect(getDoctorRun(project.id).proposal?.newCommand).toBe('rm -rf /');
  });

  it('unchanged command auto-verifies the action only', async () => {
    await diagnose(project, { ...failure });
    expect(h.run).toHaveBeenCalledWith([
      expect.objectContaining({ id: 'test', command: 'pnpm test' }),
    ]);
    expect(h.runById).not.toHaveBeenCalled();
  });

  it('Verify runs the action it checked, even if pipeline.yml changes right after the check', async () => {
    h.command = 'pnpm typecheck';
    await diagnose(project, { ...failure });
    h.afterResolve = () => {
      h.command = 'rm -rf /'; // lands between Verify's re-check and the run
    };
    const out = await verifyProposal(project);
    expect(out?.success).toBe(true);
    expect(h.runById).not.toHaveBeenCalled();
    expect(h.run).toHaveBeenCalledTimes(1);
    expect(h.run).toHaveBeenCalledWith([
      expect.objectContaining({ id: 'test', command: 'pnpm typecheck' }),
    ]);
  });
});

describe('failure card survives leaving the page mid-run (QA-156)', () => {
  it('doctor state holds the failure while the agent works and drops it once fixed', async () => {
    let finish = () => {};
    h.send.mockImplementation(async () => {
      finish = () =>
        h.doneCb?.({
          thread_id: 't1',
          final_text: 'DOCTOR_VERDICT: pipeline',
          exhausted: false,
          cancelled: false,
        });
    });
    const pending = diagnose(project, { ...failure });
    await vi.waitFor(() => expect(h.send).toHaveBeenCalled());
    // A remounted panel has no lastResult — this is all it can render from.
    const mid = getDoctorRun(project.id);
    expect(mid.status).toBe('diagnosing');
    expect(mid.failure?.actionId).toBe('test');
    finish();
    await pending;
    expect(getDoctorRun(project.id).status).toBe('fixed');
    expect(getDoctorRun(project.id).failure).toBeUndefined();
  });

  it('a red verify keeps the new failure for the card', async () => {
    const red = { ...failure, outputTail: 'still red' };
    h.run.mockResolvedValue({ success: false, steps: [], cwd: 'D:/demo', failure: red });
    await diagnose(project, { ...failure });
    expect(getDoctorRun(project.id).failure?.outputTail).toBe('still red');
  });
});
