import { describe, expect, it } from 'vitest';
import { classifyFailure, toFailure } from './failure';
import type { ActionRunResult, ActionStepResult } from './types';

function result(step: Partial<ActionStepResult>, success = false): ActionRunResult {
  return {
    success,
    cwd: 'D:/demo',
    target: step.actionId ?? 'build',
    steps: [
      {
        name: step.name ?? 'Build',
        actionId: step.actionId ?? 'build',
        runner: 'local',
        status: success ? 'success' : 'failed',
        ...step,
      },
    ],
  };
}

describe('classifyFailure', () => {
  it('flags a missing script as a pipeline failure', () => {
    const out = 'ERR_PNPM_NO_SCRIPT  Missing script: build';
    expect(classifyFailure(out, 1).kind).toBe('pipeline');
  });

  it('flags "not recognized" / exit 9009 as pipeline', () => {
    expect(
      classifyFailure("'expo' is not recognized as an internal or external command", 9009).kind
    ).toBe('pipeline');
    expect(classifyFailure('', 127).kind).toBe('pipeline');
  });

  it('flags the pnpm lockfile-policy rejection as environment', () => {
    const out =
      'ERR_PNPM_LOCKFILE_CONFIG_MISMATCH  The lockfile contains entries that the active policies reject';
    const c = classifyFailure(out, 1);
    expect(c.kind).toBe('environment');
    expect(c.hint).toMatch(/package manager|pnpm clean/i);
  });

  it('leaves an ordinary test failure unknown', () => {
    const out = 'FAIL src/todos.test.ts\n  ● expected 2 to equal 3';
    expect(classifyFailure(out, 1).kind).toBe('unknown');
  });
});

describe('toFailure', () => {
  it('returns null for a successful run', () => {
    expect(toFailure(result({}, true))).toBeNull();
  });

  it('builds a classified failure carrying the failing action id', () => {
    const f = toFailure(
      result({
        actionId: 'build',
        command: 'pnpm run build',
        exitCode: 1,
        error: 'ERR_PNPM_NO_SCRIPT  Missing script: build',
      })
    );
    expect(f).not.toBeNull();
    expect(f!.actionId).toBe('build');
    expect(f!.kind).toBe('pipeline');
    expect(f!.command).toBe('pnpm run build');
  });

  it('carries the environment hint for a lockfile-policy failure', () => {
    const f = toFailure(
      result({
        actionId: 'test',
        exitCode: 1,
        output: 'The lockfile contains entries that the active policies reject',
      })
    );
    expect(f!.kind).toBe('environment');
    expect(f!.hint).toBeTruthy();
  });

  it('truncates a huge output tail to ~8KB', () => {
    const f = toFailure(result({ exitCode: 1, output: 'x'.repeat(50_000) }));
    expect(f!.outputTail.length).toBeLessThanOrEqual(8 * 1024);
  });
});
