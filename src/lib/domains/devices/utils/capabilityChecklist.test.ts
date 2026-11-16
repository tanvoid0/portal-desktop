import { describe, it, expect } from 'vitest';
import { bulkToggleable } from './capabilityChecklist';

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
