import { describe, expect, it } from 'vitest';
import {
  PIPELINE_FILE_SCHEMA,
  buildDiagnosePrompt,
  buildGeneratePrompt,
  parseGenerated,
  parseVerdict,
} from './doctor';
import type { ActionFailure, ProjectAutomationProfile } from './types';
import type { ActionCatalog } from './resolver';

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

describe('buildGeneratePrompt', () => {
  const profile: ProjectAutomationProfile = {
    name: 'demo',
    path: 'D:/demo',
    packageManager: 'pnpm',
    frameworks: ['sveltekit'],
  };
  const catalog: Pick<ActionCatalog, 'actions' | 'workflows'> = {
    actions: [
      { id: 'build', name: 'Build', source: 'local', runner: 'local', command: 'pnpm build' },
    ],
    workflows: [
      { id: 'ci', name: 'CI', source: 'local', runner: 'local', steps: [{ action: 'build' }] },
    ],
  };
  const p = buildGeneratePrompt('Add a lint gate', profile, catalog, PIPELINE_FILE_SCHEMA);

  it('includes the requirement, stack, schema, done-line and no-commit rule', () => {
    expect(p).toContain('Add a lint gate');
    expect(p).toContain('pnpm'); // package manager
    expect(p).toContain('sveltekit'); // framework
    expect(p).toContain('build: pnpm build'); // existing action surfaced
    expect(p).toContain('version: 1'); // schema injected
    expect(p).toContain('DOCTOR_GENERATED:');
    expect(p).toContain('Do NOT run git commit');
  });

  it('handles an empty catalog', () => {
    const empty = buildGeneratePrompt('x', profile, { actions: [], workflows: [] }, 'schema');
    expect(empty).toContain('(none)');
  });
});

describe('parseGenerated', () => {
  it('extracts the id, last match wins', () => {
    expect(parseGenerated('DOCTOR_GENERATED: release')).toBe('release');
    expect(parseGenerated('doctor_generated: ci\nDOCTOR_GENERATED: lint-gate')).toBe('lint-gate');
  });
  it('returns null with no line or null input', () => {
    expect(parseGenerated('nothing')).toBeNull();
    expect(parseGenerated(null)).toBeNull();
  });
});
