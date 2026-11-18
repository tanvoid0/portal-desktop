import { describe, it, expect } from 'vitest';
import { tableList } from './tableList';

const tables = (n: number) =>
  Array.from({ length: n }, (_, i) => ({ name: `t${i}`, kind: 'table' }));

describe('tableList', () => {
  it('starts expanded up to 30 tables and collapsed past it', () => {
    expect(tableList(tables(30), '').expanded).toBe(true);
    expect(tableList(tables(31), '').expanded).toBe(false);
  });

  it('lets an explicit toggle beat the size default', () => {
    expect(tableList(tables(31), '', true).expanded).toBe(true);
    expect(tableList(tables(5), '', false).expanded).toBe(false);
  });

  it('filters by case-insensitive substring, and a live filter forces the list open', () => {
    const all = [
      ...tables(40),
      { name: 'Projects', kind: 'table' },
      { name: 'tag_projects', kind: 'view' },
    ];
    const shown = tableList(all, '  PROJ ', false);
    expect(shown.visible.map((t) => t.name)).toEqual(['Projects', 'tag_projects']);
    expect(shown.expanded).toBe(true);
    expect(tableList(all, 'zzz').visible).toEqual([]);
  });

  it('ignores a filter on a small list, which has no filter box', () => {
    const shown = tableList(tables(5), 't1');
    expect(shown.query).toBe('');
    expect(shown.visible).toHaveLength(5);
  });
});
