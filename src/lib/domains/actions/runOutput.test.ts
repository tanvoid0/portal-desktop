import { describe, expect, it } from 'vitest';
import { runRequester, toTerminalText } from './runOutput';

describe('toTerminalText', () => {
  it('keeps only the last redraw of a \\r progress line', () => {
    expect(toTerminalText('start\n10%\r50%\r100%\ndone\n')).toBe('start\n100%\ndone\n');
  });

  it('shows a mid-redraw line (chunk ending in \\r) instead of blanking it', () => {
    expect(toTerminalText('Downloading 45%\r')).toBe('Downloading 45%');
  });

  it('strips colour codes and keeps CRLF line breaks', () => {
    expect(toTerminalText('\x1b[32mok\x1b[0m\r\nnext')).toBe('ok\nnext');
  });
});

describe('runRequester', () => {
  it('is null for the user and names MCP clients', () => {
    expect(runRequester('user')).toBeNull();
    expect(runRequester(undefined)).toBeNull();
    expect(runRequester('mcp:claude')).toBe('MCP · claude');
  });
});
