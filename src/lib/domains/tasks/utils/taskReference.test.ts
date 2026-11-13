import { describe, it, expect } from 'vitest';
import { buildTaskReference, buildCoderPrompt } from './taskReference';
import type { Task } from '../types';

const base: Task = {
  id: 'abc123',
  title: 'Fix login redirect',
  status: 'in-progress',
  priority: 'high',
  type: 'Bug',
  description: 'Redirect loops after SSO.',
  tags: ['auth', 'urgent'],
  createdAt: new Date(),
  updatedAt: new Date(),
};

describe('buildTaskReference', () => {
  it('includes id, title, status label, capitalised priority and type', () => {
    const ref = buildTaskReference(base, 'portal_desktop');
    expect(ref).toContain('Portal task #abc123 — Fix login redirect');
    expect(ref).toContain('Status: In Progress · Priority: High · Type: Bug');
    expect(ref).toContain('Project: portal_desktop');
    expect(ref).toContain('Tags: auth, urgent');
    expect(ref).toContain('Redirect loops after SSO.');
  });

  it('omits optional lines and falls back for a missing description', () => {
    const ref = buildTaskReference({
      ...base,
      type: undefined,
      tags: [],
      description: undefined,
    });
    expect(ref).not.toContain('Type:');
    expect(ref).not.toContain('Project:');
    expect(ref).not.toContain('Tags:');
    expect(ref).toContain('(no description)');
  });

  it('wraps the reference as an actionable coder prompt', () => {
    const prompt = buildCoderPrompt(base);
    expect(prompt.startsWith('Portal task #abc123')).toBe(true);
    expect(prompt).toContain('Implement this task.');
  });
});
