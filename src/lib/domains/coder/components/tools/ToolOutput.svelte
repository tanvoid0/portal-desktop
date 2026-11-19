<script lang="ts">
  interface Props {
    text: string;
    failed?: boolean;
  }

  let { text, failed = false }: Props = $props();
</script>

<pre
  class="max-h-48 overflow-auto rounded p-2 text-xs {failed
    ? 'bg-destructive/10 text-destructive'
    : 'bg-muted/40'}"><code>{text}</code></pre>
