import { describe, expect, it } from 'vitest';
import { parseImportJson } from './importJson';

describe('parseImportJson', () => {
  it('rejects non-definitions', () => {
    for (const t of ['[1,2,3]', '{"foo":1}', '{"name":"a"}', 'null', '{bad'])
      expect(parseImportJson(t, 'command')).toHaveProperty('error');
  });
  it('accepts name + command', () => {
    expect(parseImportJson('{"name":"a","command":"echo"}', 'script')).toMatchObject({
      data: { name: 'a', command: 'echo', executionType: 'script' },
    });
  });
});
