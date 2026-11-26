import { describe, expect, it } from 'vitest';
import { commonDir } from './commonDir';

describe('commonDir', () => {
  it('finds the shared folder by whole segments', () => {
    expect(commonDir(['D:\\p\\app', 'd:\\p\\app-web'])).toBe('D:\\p');
    expect(commonDir(['D:\\p\\app'])).toBe('D:\\p\\app');
    expect(commonDir(['D:\\a', 'D:\\b'])).toBe('D:\\');
    expect(commonDir(['D:\\a', 'C:\\a'])).toBe('');
    expect(commonDir(['/a', '/b'])).toBe('/');
  });
});
