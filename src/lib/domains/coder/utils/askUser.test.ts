import { describe, expect, it } from 'vitest';
import { formatAnswers, parseQuestions } from './askUser.js';

describe('ask_user', () => {
  it('parses questions, object options and the bare single-question shape', () => {
    expect(
      parseQuestions(
        '{"questions":[{"question":"A?","options":["x",{"label":"y"}],"multi_select":true}]}'
      )
    ).toEqual([{ question: 'A?', options: ['x', 'y'], multiSelect: true }]);
    expect(parseQuestions('{"question":"B?","options":["x","x"]}')).toEqual([
      { question: 'B?', options: ['x'], multiSelect: false },
    ]);
    expect(parseQuestions('not json')).toEqual([]);
    expect(parseQuestions('{}')).toEqual([]);
  });

  it('formats the answer for the model', () => {
    const qs = parseQuestions('{"question":"A?","options":["x","y"]}');
    expect(formatAnswers(qs, [['x', 'y']])).toBe('User answered:\n- A? => x, y');
  });
});
