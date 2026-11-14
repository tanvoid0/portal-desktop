import { describe, expect, it } from 'vitest';
import { isForwardableCommand } from './commandName';

describe('isForwardableCommand', () => {
  it('accepts plain command names', () => {
    expect(isForwardableCommand('get_all_projects')).toBe(true);
    expect(isForwardableCommand('coder_list_threads')).toBe(true);
    expect(isForwardableCommand('mobile_status')).toBe(true);
  });
  it('rejects plugin commands and anything with punctuation', () => {
    expect(isForwardableCommand('plugin:barcode-scanner|scan')).toBe(false);
    expect(isForwardableCommand('plugin:event|listen')).toBe(false);
    expect(isForwardableCommand('some.command')).toBe(false);
    expect(isForwardableCommand('')).toBe(false);
  });
});
