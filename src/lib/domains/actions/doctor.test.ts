import { describe, expect, it } from 'vitest';
import { buildDiagnosePrompt, parseVerdict } from './doctor';
import type { ActionFailure } from './types';

describe('parseVerdict', () => {
  it.each(['app', 'pipeline', 'environment', 'flaky'])('extracts %s', (v) => {
    expect(parseVerdict(`done\nDOCTOR_VERDICT: ${v}`)).toBe(v);
  });
  it('is case-insensitive and last match wins', () => {
    expect(parseVerdict('doctor_verdict: APP\nmore\nDOCTOR_VERDICT: Flaky')).toBe('flaky');
  });
  it('returns unknown with no line or null', () => {
    expect(parseVerdict('nothing here')).toBe('unknown');
    expect(parseVerdict(null)).toBe('unknown');
  });
});

describe('buildDiagnosePrompt', () => {
  const failure: ActionFailure = {
    actionId: 'build',
    runner: 'local',
    command: 'pnpm build',
    exitCode: 1,
    outputTail: 'boom',
    cwd: 'D:/demo',
    target: 'build',
    kind: 'pipeline',
  };
  const p = buildDiagnosePrompt(failure);
  it('includes command, verdict instruction and no-commit rule', () => {
    expect(p).toContain('pnpm build');
    expect(p).toContain('DOCTOR_VERDICT:');
    expect(p).toContain('Do NOT run git commit');
  });
});
