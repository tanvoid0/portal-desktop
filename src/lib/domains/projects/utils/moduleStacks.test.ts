import { describe, expect, it } from 'vitest';
import type { ProjectModule } from '$lib/domains/projects/types';
import { describeModules, groupByProject, summarizeStacks } from './moduleStacks';

const mod = (project_id: number, stack: string, i = 0): ProjectModule => ({
  id: i,
  project_id,
  rel_path: `apps/${stack}${i}`,
  name: `${stack}${i}`,
  stack,
  role: 'app',
});

describe('moduleStacks', () => {
  const mods = [mod(1, 'flutter', 1), mod(1, 'flutter', 2), mod(1, 'nestjs', 3), mod(2, 'rust', 4)];

  it('groups by project', () => {
    expect([...groupByProject(mods).keys()]).toEqual([1, 2]);
  });

  it('counts stacks, most common first', () => {
    expect(summarizeStacks(mods.slice(0, 3)).map((s) => [s.label, s.count])).toEqual([
      ['Flutter', 2],
      ['NestJS', 1],
    ]);
  });

  it('describes and truncates', () => {
    expect(describeModules(mods.slice(0, 3))).toBe('2 Flutter · 1 NestJS');
    expect(describeModules(mods, 1)).toBe('2 Flutter · +2');
  });
});
