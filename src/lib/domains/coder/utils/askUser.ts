/** Parsing and answer formatting for the coder's `ask_user` tool. */

export interface AskQuestion {
  question: string;
  options: string[];
  multiSelect: boolean;
}

/** Tolerant: weak local models send options as objects, or a single bare question. */
export function parseQuestions(rawArgs: string): AskQuestion[] {
  let args: unknown;
  try {
    args = JSON.parse(rawArgs || '{}');
  } catch {
    return [];
  }
  const root = (args ?? {}) as Record<string, unknown>;
  const list = Array.isArray(root.questions) ? root.questions : [root];
  return list.flatMap((item) => {
    const q = (item ?? {}) as Record<string, unknown>;
    if (typeof q.question !== 'string' || !q.question.trim()) return [];
    const options = (Array.isArray(q.options) ? q.options : [])
      .map((o) => (typeof o === 'string' ? o : (o as { label?: unknown } | null)?.label))
      .filter((o): o is string => typeof o === 'string' && !!o.trim());
    return [
      {
        question: q.question,
        options: [...new Set(options)],
        multiSelect: q.multi_select === true,
      },
    ];
  });
}

/** The tool result the model reads back. */
export function formatAnswers(questions: AskQuestion[], answers: string[][]): string {
  const lines = questions.map((q, i) => `- ${q.question} => ${answers[i]?.join(', ') || '(none)'}`);
  return `User answered:\n${lines.join('\n')}`;
}
