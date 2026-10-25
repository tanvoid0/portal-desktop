<!--
	"What should I scan?" — the one control every scan tab starts with.

	Suggested locations stay visible until a scan has produced something, because
	an empty tab with a bare text box gives no clue what a valid target looks like.
-->

<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { FolderOpen, Search, Square } from '@lucide/svelte';
  import type { Location } from '../../types';

  let {
    value = $bindable(''),
    locations = [],
    busy = false,
    placeholder = 'C:\\Users\\you\\Downloads',
    scanLabel = 'Scan',
    showSuggestions = true,
    onscan,
    oncancel,
  }: {
    value?: string;
    locations?: Location[];
    busy?: boolean;
    placeholder?: string;
    scanLabel?: string;
    /** Suggestions are hidden once the tab has results to show instead. */
    showSuggestions?: boolean;
    onscan: (path?: string) => void;
    oncancel?: () => void;
  } = $props();

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') value = dir;
  }
</script>

<div class="mb-3 flex items-center gap-2">
  <Button variant="outline" onclick={pickFolder} disabled={busy}>
    <FolderOpen class="size-4" /> Choose folder
  </Button>
  <Input
    bind:value
    {placeholder}
    class="flex-1"
    onkeydown={(e) => e.key === 'Enter' && !busy && value && onscan()}
  />
  <Button onclick={() => onscan()} disabled={busy || !value}>
    <Search class="size-4" />
    {busy ? 'Working…' : scanLabel}
  </Button>
  {#if busy && oncancel}
    <Button variant="destructive" onclick={oncancel}>
      <Square class="size-4" /> Stop
    </Button>
  {/if}
</div>

{#if showSuggestions && locations.length > 0}
  <div class="mb-6">
    <div class="mb-2 text-xs tracking-wide text-muted-foreground uppercase">
      Or start from a drive or common folder
    </div>
    <div class="flex flex-wrap gap-2">
      {#each locations as loc (loc.path)}
        <Button
          variant="outline"
          size="sm"
          onclick={() => onscan(loc.path)}
          disabled={busy}
          title={loc.path}
        >
          <span class="text-muted-foreground">{loc.kind === 'drive' ? '▮' : '▸'}</span>
          {loc.label}
        </Button>
      {/each}
    </div>
  </div>
{/if}
