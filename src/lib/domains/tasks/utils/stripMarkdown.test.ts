import { describe, expect, it } from 'vitest';
import { stripMarkdown } from './stripMarkdown';

describe('stripMarkdown', () => {
  it('drops headings, emphasis, links and list markers', () => {
    const md = '## Goal\n\nFix **the** `parser` — see [docs](https://x.y).\n\n- one\n2. two';
    expect(stripMarkdown(md)).toBe('Goal Fix the parser — see docs. one two');
  });

  it('keeps code body, drops fences', () => {
    expect(stripMarkdown('```ts\nconst a = 1;\n```')).toBe('const a = 1;');
  });

  it('leaves snake_case and plain text untouched', () => {
    expect(stripMarkdown('use my_var_name here')).toBe('use my_var_name here');
  });
});
