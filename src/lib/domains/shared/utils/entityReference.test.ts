import { describe, it, expect } from 'vitest';
import { buildReference } from './entityReference';

describe('buildReference', () => {
  it('renders kind, id, title, fields and body', () => {
    const ref = buildReference({
      kind: 'task',
      id: 'abc123',
      title: 'Fix login redirect',
      fields: { Status: 'In Progress', Priority: 'High', Type: 'Bug' },
      body: 'Redirect loops after SSO.',
    });
    expect(ref).toContain('Portal task #abc123 — Fix login redirect');
    expect(ref).toContain('Status: In Progress · Priority: High · Type: Bug');
    expect(ref).toContain('Redirect loops after SSO.');
  });

  it('drops falsy fields and omits the body when undefined', () => {
    const ref = buildReference({
      kind: 'project',
      id: 7,
      title: 'portal_desktop',
      fields: { Stage: undefined, Branch: 'main' },
    });
    expect(ref).toContain('Portal project #7 — portal_desktop');
    expect(ref).toContain('Branch: main');
    expect(ref).not.toContain('Stage:');
    // No trailing blank line / body when there is none.
    expect(ref.endsWith('Branch: main')).toBe(true);
  });
});
