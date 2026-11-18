import { describe, expect, it } from 'vitest';
import { etaText, isOverrun, segmentFraction } from './runProgress';

const t0 = Date.parse('2026-01-01T00:00:00Z');
const seg = {
  stepId: 'a',
  name: 'A',
  status: 'running',
  weight: 1,
  startedAt: '2026-01-01T00:00:00Z',
  expectedMs: 10_000,
};

describe('segmentFraction', () => {
  it('uses backend fraction first', () => {
    expect(segmentFraction({ ...seg, fraction: 0.3 }, t0 + 9000)).toBe(0.3);
  });
  it('estimates from elapsed and caps at 0.95', () => {
    expect(segmentFraction(seg, t0 + 5000)).toBe(0.5);
    expect(segmentFraction(seg, t0 + 50_000)).toBe(0.95);
  });
  it('done is full, pending empty', () => {
    expect(segmentFraction({ ...seg, status: 'success' }, t0)).toBe(1);
    expect(segmentFraction({ ...seg, status: 'pending' }, t0)).toBe(0);
  });
});

describe('isOverrun', () => {
  it('flags only past expected', () => {
    expect(isOverrun(seg, t0 + 9000)).toBe(false);
    expect(isOverrun(seg, t0 + 11_000)).toBe(true);
  });
});

describe('etaText', () => {
  it('formats', () => {
    expect(etaText(null, t0)).toBe('No estimate yet');
    expect(etaText(new Date(t0 + 240_000).toISOString(), t0)).toBe('about 4 min left');
    expect(etaText(new Date(t0 + 20_000).toISOString(), t0)).toBe('about 20 sec left');
    expect(etaText(new Date(t0 - 1).toISOString(), t0)).toBe('finishing up');
  });
});
