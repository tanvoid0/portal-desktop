import prettier from 'eslint-config-prettier';
import { fileURLToPath } from 'node:url';
import { includeIgnoreFile } from '@eslint/compat';
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';
import portalPlugin from './eslint-rules/index.mjs';

const gitignorePath = fileURLToPath(new URL('./.gitignore', import.meta.url));

export default defineConfig(
  includeIgnoreFile(gitignorePath),
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    ignores: ['src-tauri/target/**', 'src-tauri/Cargo.lock', 'src-tauri/target/debug/build/**'],
  },
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      // typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
      // see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
      'no-undef': 'off',
      // Rules from logs-explorer project for better code quality
      '@typescript-eslint/no-unused-vars': [
        // This repo has a lot of intentionally unused imports/vars in router
        // pages/components. Treat as warnings so lint remains actionable.
        'warn',
        { argsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-explicit-any': 'warn',
      // This repo has many patterns where values are conditionally re-assigned.
      // Downgrade to warning so eslint stays useful without forcing massive refactors.
      'prefer-const': 'warn',
      'no-var': 'error',

      // The Svelte recommended rules are currently extremely noisy across
      // router pages. Disable/relax them to keep lint focused on real issues.
      'svelte/no-navigation-without-resolve': 'off',
      'svelte/require-each-key': 'off',
      'svelte/prefer-svelte-reactivity': 'off',

      // Additional Svelte/router noise removed for upgrade-support tidy-ups.
      'svelte/no-unused-props': 'off',
      'svelte/prefer-writable-derived': 'off',
      'svelte/no-at-html-tags': 'off',
      'svelte/no-useless-mustaches': 'off',
      'svelte/no-reactive-reassign': 'off',

      // Generic rules that are heavily triggered by terminal/regex-heavy logic.
      'no-useless-escape': 'off',
      'no-control-regex': 'off',
      'no-case-declarations': 'off',
      'no-self-assign': 'off',
      'no-prototype-builtins': 'off',
      '@typescript-eslint/no-unused-expressions': 'off',

      // Svelte 5 runes (`$derived`, `$state`, etc.) are referenced directly,
      // but this rule still enforces classic `$store` semantics.
      'svelte/require-store-reactive-access': 'off',
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: ['.svelte'],
        parser: ts.parser,
      },
    },
  },
  {
    // Vendored shadcn-svelte primitives: `shadcn add`/`update` rewrites these, so
    // anything fixed here comes straight back on the next sync.
    files: ['src/lib/components/ui/**/*.svelte'],
    rules: {
      'svelte/no-navigation-without-resolve': 'off',
      // `ref = $bindable(null)` is declared for API parity even where the
      // component never reads it back.
      'no-useless-assignment': 'off',
    },
  },
  {
    files: ['src/**/*.svelte'],
    ignores: ['src/lib/components/ui/**'],
    plugins: {
      portal: portalPlugin,
    },
    rules: {
      'portal/no-raw-ui-elements': 'error',
      // Svelte 4 syntax: `export let`, `on:click`, `<slot>`, `<svelte:fragment>`,
      // `$:`, `<svelte:component>`. Kept off the vendored ui/ primitives, which
      // `shadcn add` rewrites.
      'portal/no-legacy-syntax': 'error',
      // The other half of the Svelte 4 component contract. `$:` and `<slot>` are
      // caught above; this catches the dispatcher that replaced callback props.
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: 'svelte',
              importNames: ['createEventDispatcher'],
              message:
                'createEventDispatcher is Svelte 4. Declare a callback prop (`onEdit?: (p: Project) => void`) instead.',
            },
          ],
        },
      ],
      // `$inspect` is a debugger, not production code.
      'svelte/no-inspect': 'error',
      // `$state(new Set())` instead of `new SvelteSet()` — reactive-looking, inert.
      'svelte/no-unnecessary-state-wrap': 'error',
      // Deliberately NOT enabled: `svelte/require-event-prefix`. It flags every
      // function-typed prop, not just event callbacks, so formatters like
      // `projectLabel?: (task) => string` come back as violations. 23 hits, most
      // of them false. Callback-prop naming stays a review matter.
    },
  },
  {
    // Applies everywhere runes are valid, including the ui/ primitives.
    files: ['src/**/*.svelte', 'src/**/*.svelte.ts', 'src/**/*.ts'],
    plugins: {
      portal: portalPlugin,
    },
    rules: {
      'portal/no-derived-closure': 'error',
    },
  }
);
