<script lang="ts">
  import MarkdownFileContent from '$lib/components/ui/chat-markdown/MarkdownFileContent.svelte';
  import { isMarkdownPath } from '$lib/utils/markdownFile.js';
  import ToolOutput from './ToolOutput.svelte';
  import type { ToolBodyProps } from './types.js';

  // Read shows the result, write shows what was written.
  let { kind, args, result, failed, failedOutput }: ToolBodyProps = $props();

  const content = $derived(kind === 'write' ? String(args.content ?? '') : (result ?? ''));
</script>

{#if failed}
  <ToolOutput text={failedOutput} failed />
{:else if kind === 'write' || result !== null}
  {#if isMarkdownPath(String(args.path ?? ''))}
    <MarkdownFileContent {content} />
  {:else}
    <ToolOutput text={content} />
  {/if}
{/if}
