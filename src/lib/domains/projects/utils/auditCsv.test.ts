import { describe, expect, it } from 'vitest';
import { toCsv } from './auditCsv';
import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';

function project(overrides: Partial<DiscoveredProject> = {}): DiscoveredProject {
  return {
    path: 'production/web/app',
    abs: 'D:\\production\\web\\app',
    name: 'app',
    area: 'production',
    bucket: 'production/web',
    moduleCount: 1,
    modules: [],
    activity: 0,
    ageDays: 12,
    status: 'active',
    git: {
      repos: 1,
      branch: 'main',
      remote: 'git@host:me/app.git',
      lastCommit: null,
      commits: 3,
      ahead: 2,
      behind: 0,
      stash: 0,
      dirty: 1,
      noRemote: 0,
    },
    langs: ['JS/TS'],
    regenerable: ['node_modules'],
    regenerableBytes: 2048,
    description: null,
    risk: ['2 unpushed commits'],
    riskLevel: 'high',
    duplicateOf: [],
    ...overrides,
  } as DiscoveredProject;
}

describe('toCsv', () => {
  it('writes a header and one row per project', () => {
    const csv = toCsv([project()], { 'production/web/app': 'keep' });
    const lines = csv.split('\r\n');
    expect(lines).toHaveLength(2);
    expect(lines[0].startsWith('"path","name","abs"')).toBe(true);
    expect(lines[1]).toContain('"keep"');
    expect(lines[1]).toContain('"2048"');
  });

  it('escapes quotes and keeps commas inside one field', () => {
    const csv = toCsv([project({ name: 'the "old", copy' })], {});
    // The name must survive as a single field: quotes doubled, comma untouched.
    expect(csv).toContain('"the ""old"", copy"');
    expect(csv.split('\r\n')).toHaveLength(2);
  });

  it('leaves an unmeasured size and an absent decision blank, not zero', () => {
    const csv = toCsv([project({ regenerableBytes: -1 })], {});
    const row = csv.split('\r\n')[1];
    expect(row.endsWith('""')).toBe(true);
    expect(row).toContain('"node_modules",""');
  });
});
