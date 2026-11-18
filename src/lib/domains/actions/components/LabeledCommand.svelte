<!-- Small muted label above a monospace command block. -->
<script lang="ts">
  interface Props {
    label: string;
    command?: string;
  }

  let { label, command }: Props = $props();
</script>

<div>
  <p class="mb-1 text-xs font-medium text-muted-foreground">{label}</p>
  <code class="block rounded bg-muted px-2 py-1 text-xs break-all">
    {command || '—'}
  </code>
</div>
