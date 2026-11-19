import { describe, expect, it } from 'vitest';
import { ansiSpans, runRequester, toTerminalText } from './runOutput';

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

describe('ansiSpans', () => {
  it('turns SGR colours into classes and resets on 0', () => {
    expect(ansiSpans('\x1b[32m  ok\x1b[0m no shadows')).toEqual([
      { text: '  ok', cls: 'text-green-400' },
      { text: ' no shadows', cls: '' },
    ]);
  });

  it('combines bold with a bright colour and merges equal neighbours', () => {
    expect(ansiSpans('\x1b[1;91mFAIL\x1b[1;91m!\x1b[22m x')).toEqual([
      { text: 'FAIL!', cls: 'text-red-300 font-bold' },
      { text: ' x', cls: 'text-red-300' },
    ]);
  });

  it('consumes 256/truecolour arguments instead of reading them as codes', () => {
    expect(ansiSpans('\x1b[38;2;1;32;3mrgb\x1b[38;5;2mbase')).toEqual([
      { text: 'rgb', cls: '' },
      { text: 'base', cls: 'text-green-400' },
    ]);
  });

  it('drops non-SGR escapes and keeps the last \\r redraw with its colour', () => {
    expect(ansiSpans('\x1b[33m10%\r50%\x1b[K\r100%\r')).toEqual([
      { text: '100%', cls: 'text-yellow-300' },
    ]);
  });
});

describe('runRequester', () => {
  it('is null for the user and names MCP clients', () => {
    expect(runRequester('user')).toBeNull();
    expect(runRequester(undefined)).toBeNull();
    expect(runRequester('mcp:claude')).toBe('MCP · claude');
  });
});
