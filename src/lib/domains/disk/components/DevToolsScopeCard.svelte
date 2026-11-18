<script lang="ts">
  import type { Location } from '../types';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Boxes from '~icons/fa6-solid/boxes-stacked';
  import Container from '~icons/fa6-solid/cube';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Square from '~icons/fa6-solid/square';
  import X from '~icons/fa6-solid/xmark';

  let {
    suggestedRoots,
    customRoots,
    scanRoots,
    busy,
    ontoggleRoot,
    onpick,
    onscan,
    oncancel,
  }: {
    suggestedRoots: Location[];
    customRoots: string[];
    scanRoots: Set<string>;
    busy: boolean;
    ontoggleRoot: (path: string) => void;
    onpick: () => void;
    onscan: () => void;
    oncancel: () => void;
  } = $props();
</script>

<Card class="mb-4 gap-0 py-4">
  <CardContent class="space-y-3 px-4">
    <div class="flex flex-wrap items-center gap-2 text-sm">
      <Container class="size-4 text-muted-foreground" />
      <span class="font-medium text-foreground">Container tools</span>
      <span class="text-muted-foreground">
        Docker and Podman — stopped containers, dangling images, unused volumes and build cache.
        Always included; skipped when the tool is not installed.
      </span>
    </div>

    <div class="flex flex-wrap items-center gap-2 text-sm">
      <Boxes class="size-4 text-muted-foreground" />
      <span class="font-medium text-foreground">Project build output</span>
      <span class="text-muted-foreground">
        {scanRoots.size
          ? 'node_modules, target, dist and friends under the folders below.'
          : 'Pick a folder to include node_modules, target, dist and friends.'}
      </span>
    </div>

    <div class="flex flex-wrap gap-2">
      {#each suggestedRoots as loc (loc.path)}
        <Button
          variant={scanRoots.has(loc.path) ? 'default' : 'outline'}
          size="sm"
          onclick={() => ontoggleRoot(loc.path)}
          disabled={busy}
          title={loc.path}
        >
          {loc.label}
        </Button>
      {/each}
      {#each customRoots as path (path)}
        <Badge variant="secondary" class="gap-1 font-mono text-xs">
          {path}
          <Button
            variant="ghost"
            class="h-auto p-0"
            aria-label={`Remove ${path}`}
            onclick={() => ontoggleRoot(path)}
          >
            <X class="size-3" />
          </Button>
        </Badge>
      {/each}
      <Button variant="outline" size="sm" onclick={onpick} disabled={busy}>
        <FolderOpen class="size-4" /> Add folder…
      </Button>
    </div>

    <div class="flex items-center gap-2">
      <Button onclick={onscan} disabled={busy}>
        <Search class="size-4" />
        {busy ? 'Working…' : 'Scan'}
      </Button>
      {#if busy && scanRoots.size > 0}
        <!-- Only the project walk is cancellable; container enumeration is not. -->
        <Button variant="destructive" onclick={oncancel}>
          <Square class="size-4" /> Stop
        </Button>
      {/if}
      <span class="text-xs text-muted-foreground">
        {scanRoots.size
          ? `Containers + projects under ${scanRoots.size} folder${scanRoots.size === 1 ? '' : 's'}`
          : 'Containers only — add a folder to include projects'}
      </span>
    </div>
  </CardContent>
</Card>
