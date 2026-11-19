import { describe, expect, it } from 'vitest';
import { getToolResultStatus } from './toolCallDisplay';
import { normalizeTool } from './toolKinds';

describe('exit-code failure parsing', () => {
  it('only applies to Portal run_command', () => {
    expect(getToolResultStatus('commandExecution', 'exit 2\n')).toBe('success');
    expect(getToolResultStatus('run_command', 'exit 2\nboom')).toBe('failed');
  });
});

describe('normalizeTool', () => {
  it('maps Bash and run_command to command with canonical args', () => {
    expect(normalizeTool('Bash', { command: 'ls', description: 'x' })).toEqual({
      kind: 'command',
      args: { command: 'ls' },
    });
    expect(normalizeTool('run_command', { command: 'ls' }).kind).toBe('command');
    expect(normalizeTool('commandExecution', { command: 'ls' }).kind).toBe('command');
  });

  it('maps file_path and path to canonical path', () => {
    expect(normalizeTool('Read', { file_path: 'a.ts' }).args.path).toBe('a.ts');
    expect(normalizeTool('read_file', { path: 'a.ts' }).args.path).toBe('a.ts');
  });

  it('maps the remaining native names', () => {
    expect(normalizeTool('TodoWrite', { todos: [] }).kind).toBe('todo');
    expect(normalizeTool('Task', { description: 'd', prompt: 'p' }).kind).toBe('subagent');
    expect(normalizeTool('MultiEdit', { file_path: 'a', edits: [{}] })).toMatchObject({
      kind: 'multi-edit',
      args: { path: 'a', edits: [{}] },
    });
    expect(normalizeTool('Grep', { pattern: 'x' }).args.query).toBe('x');
    expect(normalizeTool('WebFetch', { url: 'u' }).kind).toBe('web');
    expect(normalizeTool('delegate_task', { goal: 'g' }).args.description).toBe('g');
  });

  it('falls back for unknown tools and keeps their args', () => {
    expect(normalizeTool('mystery', { a: 1 })).toEqual({ kind: 'fallback', args: { a: 1 } });
    expect(normalizeTool('x', null).args).toEqual({});
  });
});
