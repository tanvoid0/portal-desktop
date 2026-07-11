/** @type {import('eslint').Rule.RuleModule} */
const rule = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow raw HTML UI elements outside shadcn primitive files; use $lib/components/ui components",
    },
    messages: {
      rawElement:
        "Use shadcn-svelte {{component}} from $lib/components/ui instead of raw <{{tag}}>. See .cursor/rules/shadcn-ui.mdc",
    },
    schema: [],
  },
  create(context) {
    const filename = context.filename.replace(/\\/g, "/");

    if (filename.includes("/src/lib/components/ui/")) {
      return {};
    }

    if (!filename.endsWith(".svelte")) {
      return {};
    }

    const source = context.sourceCode.getText();

    /** @param {string} tag @param {string} component */
    function report(tag, component, index) {
      const line = source.slice(0, index).split("\n").length;
      const column = index - source.lastIndexOf("\n", index);
      context.report({
        loc: { line, column: Math.max(1, column) },
        messageId: "rawElement",
        data: { tag, component },
      });
    }

    const patterns = [
      { regex: /<button\b/g, tag: "button", component: "Button" },
      { regex: /<textarea\b/g, tag: "textarea", component: "Textarea" },
      { regex: /<select\b/g, tag: "select", component: "Select" },
      {
        regex: /<input\b/g,
        tag: "input",
        component: "Input",
        allow: (match) => {
          const snippet = source.slice(match.index, match.index + 120);
          return /\btype\s*=\s*["'](?:file|hidden)["']/i.test(snippet);
        },
      },
      {
        regex: /<label\b/g,
        tag: "label",
        component: "Label",
        allow: (match) => {
          const snippet = source.slice(match.index, match.index + 400);
          return /<Checkbox\b/.test(snippet);
        },
      },
    ];

    return {
      Program() {
        for (const { regex, tag, component, allow } of patterns) {
          regex.lastIndex = 0;
          let match;
          while ((match = regex.exec(source)) !== null) {
            if (allow?.(match)) continue;
            report(tag, component, match.index);
          }
        }
      },
    };
  },
};

export default rule;
