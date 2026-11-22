import { describe, it, expect } from 'vitest';
import { bulkToggleable, withoutSensitive } from './capabilityChecklist';

describe('bulkToggleable', () => {
  it('excludes sensitive capabilities from the bulk-select set', () => {
    const names = bulkToggleable([
      { name: 'projects', is_sensitive: false },
      { name: 'terminal.exec', is_sensitive: true },
      { name: 'agents.exec', is_sensitive: true },
    ]);
    expect(names).toEqual(['projects']);
  });

  it('is empty when every capability is sensitive', () => {
    expect(bulkToggleable([{ name: 'remote.input', is_sensitive: true }])).toEqual([]);
  });
});

describe('withoutSensitive', () => {
  const caps = [
    { name: 'projects', is_sensitive: false },
    { name: 'terminal.exec', is_sensitive: true },
  ];

  it('drops sensitive names from a requested set', () => {
    expect(withoutSensitive(['projects', 'terminal.exec'], caps)).toEqual(['projects']);
  });

  it('drops names the registry does not know about', () => {
    expect(withoutSensitive(['projects', 'unknown.cap'], caps)).toEqual(['projects']);
  });

  it('grants nothing while the capability list is empty (query not loaded)', () => {
    expect(withoutSensitive(['terminal.exec'], [])).toEqual([]);
  });
});
