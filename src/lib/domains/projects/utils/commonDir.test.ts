import { describe, expect, it } from 'vitest';
import { commonDir, isTooBroad } from './commonDir';

describe('commonDir', () => {
  it('finds the shared folder by whole segments', () => {
    expect(commonDir(['D:\\p\\app', 'd:\\p\\app-web'])).toBe('D:\\p');
    expect(commonDir(['D:\\p\\app'])).toBe('D:\\p\\app');
    expect(commonDir(['D:\\a', 'D:\\b'])).toBe('D:\\');
    expect(commonDir(['D:\\a', 'C:\\a'])).toBe('');
    expect(commonDir(['/a', '/b'])).toBe('/');
  });
});

describe('commonDir edge paths', () => {
  it('has no answer for UNC paths', () => {
    expect(commonDir(['\\\\srv\\share\\a', '\\\\srv\\share\\b'])).toBe('');
    expect(commonDir(['//srv/share/a', '//srv/share/b'])).toBe('');
  });

  it('is case-sensitive outside drive-letter paths', () => {
    expect(commonDir(['/w/App/a', '/w/app/b'])).toBe('/w');
  });
});

describe('isTooBroad', () => {
  it('refuses roots, home and above, but not folders under or beside home', () => {
    const home = 'C:\\Users\\me';
    for (const dir of ['C:\\', 'D:\\', 'c:\\users', 'C:\\Users\\Me\\'])
      expect(isTooBroad(dir, home)).toBe(true);
    for (const dir of ['C:\\Users\\me\\code', 'D:\\p', 'C:\\Users\\m'])
      expect(isTooBroad(dir, home)).toBe(false);
    expect(isTooBroad('/', '/home/me')).toBe(true);
    expect(isTooBroad('/home', '/home/me')).toBe(true);
    expect(isTooBroad('/srv/code', '/home/me')).toBe(false);
  });
});
