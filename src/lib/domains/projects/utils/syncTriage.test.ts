import { describe, expect, it } from 'vitest';
import { afterApply, commitEverything, emptyPlan, parseTriage } from './syncTriage';
import type { SyncRepoFacts } from '../generated/SyncRepoFacts';

const facts: SyncRepoFacts = {
  branch: 'main',
  ahead: 0,
  behind: 0,
  files: [
    { path: 'src/a.ts', status: 'M', secret: false },
    { path: 'src/b.ts', status: '??', secret: false },
    { path: 'src/.env', status: '??', secret: true },
  ],
  filesTotal: 3,
  diff: '',
  recentSubjects: [],
  fingerprint: 'x',
  pushOk: false,
};

describe('parseTriage', () => {
  it('keeps only changed, non-secret files and spells out folders', () => {
    const reply = `<think>hmm</think>\`\`\`json
      {"summary":"s","commits":[
        {"message":"feat: a","files":["src/","src/a.ts","nope.ts"]},
        {"message":"empty","files":["src/.env"]}],
       "ignore":["*","!keep","dist/"],"questions":[]}\`\`\``;
    expect(parseTriage(reply, facts)).toEqual({
      summary: 's',
      commits: [{ message: 'feat: a', files: ['src/a.ts', 'src/b.ts'] }],
      ignore: ['dist/'],
      questions: [],
    });
  });

  it('holds the plan back while there are questions', () => {
    const reply = JSON.stringify({
      summary: 's',
      commits: [{ message: 'm', files: ['src/a.ts'] }],
      questions: [{ question: 'Commit b?', options: ['Yes', 'No'] }],
    });
    const parsed = parseTriage(reply, facts);
    expect(parsed?.commits).toEqual([]);
    expect(parsed?.questions).toEqual([
      { question: 'Commit b?', options: ['Yes', 'No'], answer: '' },
    ]);
  });

  it('rejects a reply with nothing usable', () => {
    expect(parseTriage('I cannot help', facts)).toBeNull();
    expect(parseTriage('{"commits": "all"}', facts)).toBeNull();
  });

  it('never puts a secret in the commit-everything fallback', () => {
    expect(commitEverything(facts)[0].files).toEqual(['src/a.ts', 'src/b.ts']);
  });
});

describe('afterApply', () => {
  it('clears the uncommitted mark and flags the new commit as unpushed', () => {
    const row = afterApply(
      {
        id: 1,
        name: 'p',
        path: '/p',
        ahead: 0,
        behind: 0,
        dirty: 1,
        untracked: 0,
        steps: [
          { step: 'push', status: 'skipped', detail: 'nothing to push' },
          { step: 'mark', status: 'attention', detail: '1 modified, 0 untracked' },
        ],
      },
      emptyPlan(),
      { done: [], error: null, ahead: 1, behind: 0, dirty: 0, untracked: 0 }
    );
    expect(row.steps.map((s) => [s.step, s.status])).toEqual([
      ['push', 'attention'],
      ['mark', 'ok'],
    ]);
  });
});
