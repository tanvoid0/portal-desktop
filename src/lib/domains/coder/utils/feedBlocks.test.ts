import { describe, expect, it } from 'vitest';
import { buildActivitySummary } from './feedBlocks.js';
import type { ToolCall } from '../types.js';

function call(name: string): ToolCall {
  return {
    id: `c-${name}-${Math.random()}`,
    type: 'function',
    function: { name, arguments: '{}' },
  };
}

/** `summary(...)` as one flat string, the way the feed renders it. */
function phrase(tools: Array<{ call: ToolCall; result: string | null }>): string {
  return buildActivitySummary(tools)
    .parts.map((p) => `${p.verb}${p.rest}`)
    .join(', ');
}

describe('buildActivitySummary', () => {
  it('counts successful calls', () => {
    expect(
      phrase([
        { call: call('write_file'), result: 'ok' },
        { call: call('write_file'), result: 'ok' },
      ])
    ).toBe('Created 2 files');
  });

  // QA-084: two `write_file` calls that both returned `Error: missing arg: path`
  // were reported as "Created 2 files".
  it('does not count failed calls as work done', () => {
    const summary = buildActivitySummary([
      { call: call('write_file'), result: 'Error: missing arg: path' },
      { call: call('write_file'), result: 'Error: missing arg: path' },
    ]);
    expect(summary.hasFailed).toBe(true);
    expect(summary.parts.map((p) => `${p.verb}${p.rest}`).join(', ')).toBe(
      'Failed creating 2 files'
    );
  });

  it('reports the failures alongside the successes', () => {
    expect(
      phrase([
        { call: call('write_file'), result: 'ok' },
        { call: call('write_file'), result: 'Error: missing arg: path' },
      ])
    ).toBe('Created a file (1 failed)');
  });

  it('still reports in-progress calls as active', () => {
    const summary = buildActivitySummary([{ call: call('read_file'), result: null }]);
    expect(summary.isRunning).toBe(true);
    expect(summary.hasFailed).toBe(false);
    expect(summary.parts.map((p) => `${p.verb}${p.rest}`).join(', ')).toBe('Reading a file');
  });

  it('keeps a failed tool from hiding a different tool that worked', () => {
    expect(
      phrase([
        { call: call('read_file'), result: 'contents' },
        { call: call('write_file'), result: 'Error: missing arg: path' },
      ])
    ).toBe('Failed creating a file, Read a file');
  });
});
