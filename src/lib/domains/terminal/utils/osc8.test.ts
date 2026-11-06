import { describe, expect, it } from 'vitest';
import { injectOsc8Links } from './osc8';

describe('injectOsc8Links', () => {
  it('leaves OSC window-title sequences intact', () => {
    const title = '\x1b]0;C:\WINDOWS\System32\WindowsPowerShell\v1.0\powershell.exe\x07';
    expect(injectOsc8Links(title, '').transformed).toBe(title);
  });
});
