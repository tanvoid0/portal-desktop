import { describe, expect, it } from 'vitest';
import { pressureClass, pressureTextClass, usedPct } from './utils';

describe('pressureTextClass', () => {
  it('steps from green to amber at 75 and red at 90', () => {
    expect(pressureTextClass(74.9)).toBe('text-status-success');
    expect(pressureTextClass(75)).toBe('text-status-warning');
    expect(pressureTextClass(89.9)).toBe('text-status-warning');
    expect(pressureTextClass(90)).toBe('text-status-error');
  });
});

describe('usedPct', () => {
  it('is the used share of the total', () => {
    expect(usedPct(50, 200)).toBe(25);
    expect(usedPct(200, 200)).toBe(100);
  });

  it('is 0, never NaN, for an empty or unreadable total', () => {
    expect(usedPct(0, 0)).toBe(0);
    expect(usedPct(10, 0)).toBe(0);
    expect(usedPct(10, Number.NaN)).toBe(0);
    expect(usedPct(Number.NaN, 100)).toBe(0);
  });

  it('clamps into 0-100', () => {
    expect(usedPct(300, 200)).toBe(100);
    expect(usedPct(-5, 200)).toBe(0);
  });
});

describe('pressureClass', () => {
  it('steps from default to amber at 75 and red at 90', () => {
    expect(pressureClass(74.9)).toBe('');
    expect(pressureClass(75)).toContain('bg-status-warning');
    expect(pressureClass(89.9)).toContain('bg-status-warning');
    expect(pressureClass(90)).toContain('bg-status-error');
    expect(pressureClass(100)).toContain('bg-status-error');
  });
});
