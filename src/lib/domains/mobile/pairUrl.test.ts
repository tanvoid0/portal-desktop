import { describe, expect, it } from 'vitest';
import { parsePairUrl } from './pairUrl';

describe('parsePairUrl', () => {
  it('parses host, port and token', () => {
    expect(parsePairUrl('portalremote://192.168.1.5:18420/abc-123_X')).toEqual({
      host: '192.168.1.5',
      port: 18420,
      token: 'abc-123_X',
    });
  });
  it('rejects malformed input', () => {
    expect(parsePairUrl('https://192.168.1.5:18420/abc')).toBeNull();
    expect(parsePairUrl('portalremote://192.168.1.5/abc')).toBeNull();
    expect(parsePairUrl('portalremote://192.168.1.5:99999/abc')).toBeNull();
    expect(parsePairUrl('portalremote://192.168.1.5:18420/')).toBeNull();
  });
});
