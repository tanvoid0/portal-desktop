<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import Trash2 from '~icons/fa6-solid/trash';
  import ListIcon from '~icons/fa6-solid/list';
  import type { EnvRow } from '../types';
  import { hasMultipleSegments, segments } from '../utils/envSegments';

  interface Props {
    row: EnvRow;
    expanded: boolean;
    ontoggle: () => void;
    onupdate: (patch: Partial<EnvRow>) => void;
    onremove: () => void;
  }
  let { row, expanded, ontoggle, onupdate, onremove }: Props = $props();
</script>

<div class="grid gap-3 rounded-lg border p-3 md:grid-cols-[1fr_1.5fr_auto]">
  <div>
    <Label for="name-{row.id}" class="mb-1.5 block text-xs">Name</Label>
    <Input
      id="name-{row.id}"
      value={row.name}
      disabled={!row.isNew}
      placeholder="VARIABLE_NAME"
      oninput={(e) => onupdate({ name: (e.currentTarget as HTMLInputElement).value })}
    />
  </div>
  <div>
    <div class="mb-1.5 flex items-center justify-between">
      <Label for="value-{row.id}" class="block text-xs">Value</Label>
      {#if hasMultipleSegments(row.value)}
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5 {expanded ? 'text-primary' : ''}"
          onclick={ontoggle}
          aria-pressed={expanded}
          aria-label={expanded ? 'Collapse to single line' : 'Split ; separated value into lines'}
          title={expanded ? 'Collapse to single line' : 'Split into one entry per line'}
        >
          <ListIcon class="h-3 w-3" />
        </Button>
      {/if}
    </div>
    {#if expanded}
      <Textarea
        id="value-{row.id}"
        value={segments(row.value).join('\n')}
        placeholder="value"
        rows={Math.min(Math.max(segments(row.value).length, 2), 10)}
        class="font-mono text-sm"
        oninput={(e) =>
          onupdate({
            value: (e.currentTarget as HTMLTextAreaElement).value.split('\n').join(';'),
          })}
      />
    {:else}
      <Input
        id="value-{row.id}"
        value={row.value}
        placeholder="value"
        oninput={(e) => onupdate({ value: (e.currentTarget as HTMLInputElement).value })}
      />
    {/if}
  </div>
  <div class="flex items-end">
    <Button variant="ghost" size="icon" onclick={onremove} aria-label="Remove variable">
      <Trash2 class="h-4 w-4" />
    </Button>
  </div>
</div>
