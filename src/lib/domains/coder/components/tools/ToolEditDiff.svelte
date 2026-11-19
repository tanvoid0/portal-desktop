<script lang="ts">
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  // Edit and multi-edit as one unified diff: every old line red, every new line green.
  let { kind, args, failed, failedOutput }: ToolBodyProps = $props();

  const rows = $derived.by(() => {
    const edits = (kind === 'multi-edit' ? (args.edits as unknown[]) : [args]) as Array<
      Record<string, unknown>
    >;
    const out: Array<{ sign: '-' | '+'; line: string }> = [];
    for (const e of edits) {
      for (const [sign, key] of [
        ['-', 'old_string'],
        ['+', 'new_string'],
      ] as const) {
        const text = String(e[key] ?? '');
        if (text) for (const line of text.split('\n')) out.push({ sign, line });
      }
    }
    return out;
  });
</script>

{#if failed}
  <ToolOutput text={failedOutput} failed />
{:else}
  <pre class="max-h-64 overflow-auto rounded bg-muted/40 py-1 text-xs"><code
      >{#each rows as row, i (i)}<span
          class="block px-2 {row.sign === '-'
            ? 'bg-destructive/10 text-destructive'
            : 'bg-green-500/10 text-green-700 dark:text-green-400'}">{row.sign} {row.line}</span
        >{/each}</code
    ></pre>
{/if}
