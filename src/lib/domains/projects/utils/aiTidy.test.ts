import { describe, expect, it } from 'vitest';
import { buildFacts, holdsUniqueWork, parseSuggestions, type TidyRow } from './aiTidy.js';

const clean: TidyRow = { id: 1, name: 'clean', has_remote: true };
const unpushed: TidyRow = { id: 2, name: 'unpushed', has_remote: true, unpushed_commits: 3 };
const noRemote: TidyRow = { id: 3, name: 'local-only', has_remote: false };
const rows = [clean, unpushed, noRemote];

describe('holdsUniqueWork', () => {
  it('is false only when pushed, clean and with a remote', () => {
    expect(holdsUniqueWork(clean)).toBe(false);
    expect(holdsUniqueWork(unpushed)).toBe(true);
    expect(holdsUniqueWork(noRemote)).toBe(true);
    expect(holdsUniqueWork({ ...clean, stash_count: 1 })).toBe(true);
    expect(holdsUniqueWork({ ...clean, has_uncommitted_changes: true })).toBe(true);
  });
});

describe('buildFacts', () => {
  it('turns timestamps into idle days and tolerates missing ones', () => {
    const now = Date.parse('2026-11-06T00:00:00Z');
    const facts = JSON.parse(
      buildFacts(
        [
          { ...clean, last_activity: '2026-10-07T00:00:00Z' },
          { ...unpushed, last_activity: null },
          { ...noRemote, last_activity: 'garbage' },
        ],
        now
      )
    );
    expect(facts.map((f: { days_idle: number | null }) => f.days_idle)).toEqual([30, null, null]);
  });
});

describe('parseSuggestions', () => {
  it('downgrades delete to review when the project holds unique work', () => {
    const reply = JSON.stringify([
      { id: 1, action: 'delete', reason: 'redundant' },
      { id: 2, action: 'delete', reason: 'old' },
      { id: 3, action: 'delete', reason: 'old' },
    ]);
    const out = parseSuggestions(reply, rows);
    expect(out.map((s) => [s.id, s.action, s.downgraded])).toEqual([
      [1, 'delete', false],
      [2, 'review', true],
      [3, 'review', true],
    ]);
  });

  it('finds the array inside prose or a code fence', () => {
    const reply = 'Here you go:\n```json\n[{"id":1,"action":"keep","reason":"active"}]\n```';
    expect(parseSuggestions(reply, rows)).toHaveLength(1);
  });

  it('drops unknown ids, unknown actions and repeated ids', () => {
    const reply = JSON.stringify([
      { id: 99, action: 'keep' },
      { id: 1, action: 'nuke' },
      { id: 1, action: 'keep' },
      { id: 1, action: 'archive' },
    ]);
    const out = parseSuggestions(reply, rows);
    expect(out).toHaveLength(1);
    expect(out[0].action).toBe('keep');
  });

  it('returns nothing for empty or malformed replies', () => {
    expect(parseSuggestions('', rows)).toEqual([]);
    expect(parseSuggestions('[not json', rows)).toEqual([]);
    expect(parseSuggestions('{"id":1}', rows)).toEqual([]);
  });
});
