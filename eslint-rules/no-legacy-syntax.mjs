/**
 * Bans Svelte 4 syntax in this runes-only codebase.
 *
 * Every form below is something an LLM writes by default — training data skews
 * hard toward Svelte 4 — and that a human reviewer currently has to catch by
 * eye. Each one is mechanically detectable, so it belongs in lint rather than
 * in CLAUDE.md, where it is advice an agent may or may not follow.
 *
 * Deliberately not covered: `writable()` for server state. CLAUDE.md bans it
 * for server data only, and "is this server data" is not decidable from the
 * AST. Reviewers keep that one.
 */

/** @type {import('eslint').Rule.RuleModule} */
export default {
  meta: {
    type: 'problem',
    docs: {
      description: 'disallow Svelte 4 syntax in a Svelte 5 runes codebase',
    },
    schema: [],
    messages: {
      exportLet:
        '`export let` is Svelte 4. Declare an `interface Props` and destructure `$props()`.',
      eventDirective: '`on:{{name}}` is Svelte 4. Use the plain `on{{name}}` attribute.',
      slot: '`<slot>` is Svelte 4. Take a `children` snippet and `{@render children?.()}`.',
      fragment: '`<svelte:fragment>` is Svelte 4. Declare a `{#snippet}` instead.',
      reactiveStatement:
        '`$:` is Svelte 4. Use `$derived(expr)` for an expression, `$derived.by(() => {...})` for a block, or `$effect` for a side effect.',
      dynamicComponent:
        '`<svelte:component>` is deprecated in Svelte 5 — components are dynamic by default. Render the component value directly.',
    },
  },

  create(context) {
    return {
      // export let a = 1;  →  let { a = 1 }: Props = $props();
      'ExportNamedDeclaration > VariableDeclaration[kind="let"]'(node) {
        context.report({ node, messageId: 'exportLet' });
      },

      // on:click={fn}  →  onclick={fn}
      SvelteDirective(node) {
        if (node.kind !== 'EventHandler') return;
        context.report({
          node,
          messageId: 'eventDirective',
          data: { name: node.key?.name?.name ?? 'event' },
        });
      },

      // $: x = y  →  const x = $derived(y)
      SvelteReactiveStatement(node) {
        context.report({ node, messageId: 'reactiveStatement' });
      },

      SvelteElement(node) {
        const name = node.name?.name;
        if (name === 'slot') context.report({ node, messageId: 'slot' });
        else if (name === 'svelte:fragment') context.report({ node, messageId: 'fragment' });
        else if (name === 'svelte:component')
          context.report({ node, messageId: 'dynamicComponent' });
      },
    };
  },
};
