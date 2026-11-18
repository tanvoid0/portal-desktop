<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    open: boolean;
  }
  let { open = $bindable() }: Props = $props();

  const groups: { title: string; rows: [string, string][] }[] = [
    {
      title: 'Navigation & Actions',
      rows: [
        ['Create new task', 'Ctrl+N'],
        ['Open filters', 'Ctrl+F'],
        ['Focus search', 'Ctrl+K'],
        ['Toggle multi-select', 'Ctrl+M'],
      ],
    },
    {
      title: 'Multi-Select Mode',
      rows: [
        ['Select all tasks', 'Ctrl+A'],
        ['Delete selected', 'Delete'],
      ],
    },
    {
      title: 'General',
      rows: [
        ['Show shortcuts', 'Ctrl+H'],
        ['Close modal', 'Esc'],
      ],
    },
  ];
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[80vh] max-w-2xl overflow-hidden">
    <Dialog.Header class="pb-3">
      <Dialog.Title class="text-lg">Keyboard Shortcuts</Dialog.Title>
    </Dialog.Header>

    <div class="space-y-6 overflow-y-auto px-6 pb-6">
      <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
        <div class="space-y-4">
          {@render group(groups[0])}
        </div>
        <div class="space-y-4">
          {@render group(groups[1])}
          {@render group(groups[2])}
        </div>
      </div>

      <div class="divider-edge-t divider-edge-full pt-4">
        <div class="flex items-center justify-end">
          <Button onclick={() => (open = false)}>Got it</Button>
        </div>
      </div>
    </div>
  </Dialog.Content>
</Dialog.Root>

{#snippet group(g: (typeof groups)[number])}
  <h3 class="font-semibold text-foreground">{g.title}</h3>
  <div class="space-y-2">
    {#each g.rows as [label, keys] (label)}
      <div class="flex items-center justify-between">
        <span class="text-sm">{label}</span>
        <kbd class="rounded bg-muted px-2 py-1 text-xs">{keys}</kbd>
      </div>
    {/each}
  </div>
{/snippet}
