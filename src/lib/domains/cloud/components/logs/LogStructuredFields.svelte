<!-- Logger / request / HTTP fields extracted from a structured (JSON) log line -->
<script lang="ts">
  interface Props {
    fields: Record<string, any>;
  }
  let { fields }: Props = $props();

  const labelClass = 'font-medium text-slate-600 dark:text-slate-400';
  const groups = $derived([
    [
      ['Logger:', fields.logger, ''],
      ['Request ID:', fields.requestId, 'font-mono'],
      ['Trace ID:', fields.traceId, 'font-mono'],
    ],
    [
      ['Method:', fields.method, ''],
      ['URL:', fields.url, 'font-mono'],
      ['Status:', fields.statusCode, ''],
    ],
  ]);
</script>

<div class="space-y-1 text-xs">
  {#each groups as group}
    {#if group.some(([, value]) => value)}
      <div class="flex flex-wrap items-center gap-2 space-x-4">
        {#each group as [label, value, valueClass]}
          {#if value}
            <div>
              <span class={labelClass}>{label}</span>
              <span class="ml-1 {valueClass}">{value}</span>
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  {/each}
</div>
