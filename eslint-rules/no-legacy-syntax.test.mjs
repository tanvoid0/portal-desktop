import { describe, expect, it } from 'vitest';
import { Linter } from 'eslint';
import * as svelteParser from 'svelte-eslint-parser';
import rule from './no-legacy-syntax.mjs';

const linter = new Linter();

// `files` is required: ESLint's default config matches .js only, so without it
// every snippet comes back as "No matching configuration found".
const config = [
  {
    files: ['**/*.svelte'],
    plugins: { portal: { rules: { 'no-legacy-syntax': rule } } },
    languageOptions: { parser: svelteParser },
    rules: { 'portal/no-legacy-syntax': 'error' },
  },
];

/** @param {string} code */
const ids = (code) => linter.verify(code, config, 'T.svelte').map((m) => m.messageId);

describe('portal/no-legacy-syntax', () => {
  // Each case pins the AST node name the rule hangs off. If svelte-eslint-parser
  // renames one, the rule goes silently dead — that is what this guards.
  it.each([
    ['export let', '<script>export let a = 1;</script>', 'exportLet'],
    [
      'on: directive',
      '<script>function f() {}</script><button on:click={f}>x</button>',
      'eventDirective',
    ],
    ['reactive statement', '<script>let a = 1; $: b = a * 2;</script>{b}', 'reactiveStatement'],
    ['slot', '<slot />', 'slot'],
    ['svelte:fragment', '<svelte:fragment slot="a">x</svelte:fragment>', 'fragment'],
    [
      'svelte:component',
      '<script>let C = null;</script><svelte:component this={C} />',
      'dynamicComponent',
    ],
  ])('flags %s', (_label, code, messageId) => {
    expect(ids(code)).toContain(messageId);
  });

  it('leaves runes syntax alone', () => {
    // Plain JS on purpose: typing the Props interface would need the TS parser
    // layered in, and the rule is pure syntax.
    const code = `<script>
      let { a = 1, children } = $props();
      const double = $derived(a * 2);
      let open = $state(false);
      function f() { open = !open; }
    </script>
    <button onclick={f}>{double}</button>
    {@render children?.()}`;
    expect(ids(code)).toEqual([]);
  });

  it('allows `export const`, which is still valid in runes mode', () => {
    expect(ids('<script module>export const x = 1;</script>')).toEqual([]);
  });
});
