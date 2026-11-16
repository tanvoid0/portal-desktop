import { describe, expect, it } from 'vitest';
import { filterNavForCaps } from './navCaps';

const sections = [{ title: 'AI', items: [{ title: 'AI', url: '/ai' }] }];
const aiUrl = (caps: string[]) => filterNavForCaps(sections, caps)[0]?.items[0]?.url;

describe('filterNavForCaps /ai entry', () => {
  it('lands on the Portal workspace when coder is served', () => {
    expect(aiUrl(['coder', 'agents'])).toBe('/ai/coder');
  });

  it('lands on the Claude Code tab when only agents is served', () => {
    expect(aiUrl(['agents'])).toBe('/ai/coder/claude-code');
  });

  it('drops the entry when neither is served', () => {
    expect(aiUrl(['github'])).toBeUndefined();
  });
});
