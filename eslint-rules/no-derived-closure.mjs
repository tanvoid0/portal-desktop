/** @type {import('eslint').Rule.RuleModule} */
const rule = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow $derived(() => ...), which memoizes the closure instead of the value",
    },
    messages: {
      derivedClosure:
        "$derived(() => ...) makes this a function — every call site re-runs the body, so nothing is memoized. Use $derived.by(() => ...) for a block body, or $derived(expr) for a single expression.",
    },
    schema: [],
  },
  create(context) {
    const filename = context.filename.replace(/\\/g, "/");

    if (!/\.(svelte|svelte\.ts|ts)$/.test(filename)) {
      return {};
    }

    const source = context.sourceCode.getText();

    return {
      Program() {
        // Matches `$derived(()` and `$derived((): Type =>`, but not `$derived.by(`.
        const pattern = /\$derived\s*\(\s*\(\s*\)\s*(?::[^=]+)?=>/g;
        let match;

        while ((match = pattern.exec(source)) !== null) {
          const index = match.index;
          const line = source.slice(0, index).split("\n").length;
          const column = index - source.lastIndexOf("\n", index);

          context.report({
            loc: { line, column: Math.max(1, column) },
            messageId: "derivedClosure",
          });
        }
      },
    };
  },
};

export default rule;
