<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import { Label } from '$lib/components/ui/label';
  import { Progress } from '$lib/components/ui/progress';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import X from '~icons/fa6-solid/xmark';
  import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';

  interface Props {
    roots: { name: string; path: string }[];
    scanning: boolean;
    loadingState: boolean;
    progress: DiscoveryProgress | null;
    error: string | null;
    importSummary: string | null;
    /**
     * Roots and scan options are set up once and then touched monthly, so the
     * form stays folded behind the Scan button. The parent opens it on mount
     * when there is nothing to scan.
     */
    scanOpen: boolean;
    newRootPath: string;
    measureSizes: boolean;
    markMissing: boolean;
    onScan: () => void;
    onCancel: () => void;
    onAddRoot: () => void;
    onRemoveRoot: (path: string) => void;
    onPersist: () => void;
  }
  let {
    roots,
    scanning,
    loadingState,
    progress,
    error,
    importSummary,
    scanOpen = $bindable(),
    newRootPath = $bindable(),
    measureSizes = $bindable(),
    markMissing = $bindable(),
    onScan,
    onCancel,
    onAddRoot,
    onRemoveRoot,
    onPersist,
  }: Props = $props();

  const percent = $derived(
    progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0
  );
</script>

<section class="space-y-2">
  <div class="flex gap-2">
    <Button class="flex-1" onclick={onScan} disabled={scanning || !roots.length}>
      <RefreshCw class={`mr-2 h-4 w-4 ${scanning ? 'animate-spin' : ''}`} />
      {scanning ? 'Scanning…' : 'Scan and import'}
    </Button>
    {#if scanning}
      <Button variant="ghost" onclick={onCancel}>Cancel</Button>
    {/if}
  </div>

  <Button
    variant="ghost"
    class="h-auto w-full justify-start gap-1 px-1 py-1 text-xs text-muted-foreground"
    aria-expanded={scanOpen}
    onclick={() => (scanOpen = !scanOpen)}
  >
    {#if scanOpen}
      <ChevronDown class="h-3 w-3" />
    {:else}
      <ChevronRight class="h-3 w-3" />
    {/if}
    <FolderOpen class="h-3 w-3" />
    {loadingState
      ? 'Loading saved folders…'
      : `${roots.length} ${roots.length === 1 ? 'folder' : 'folders'} to scan`}
  </Button>

  {#if scanOpen}
    <div class="space-y-3">
      <div class="flex flex-wrap gap-2">
        {#each roots as root (root.path)}
          <Badge variant="secondary" class="max-w-full gap-1 font-mono text-xs">
            <span class="truncate" title={root.path}>{root.path}</span>
            <!-- 24px is the WCAG 2.2 floor for a pointer target; the glyph
                 stays 12px, only the hit area grows. -->
            <Button
              variant="ghost"
              class="-mr-1 h-6 w-6 shrink-0 rounded-full p-0"
              aria-label={`Remove ${root.path}`}
              onclick={() => onRemoveRoot(root.path)}
              disabled={scanning}
            >
              <X class="h-3 w-3" />
            </Button>
          </Badge>
        {:else}
          <p class="text-sm text-muted-foreground">
            No folders yet — add the drive or directory your projects live under.
          </p>
        {/each}
      </div>

      <div class="flex gap-2">
        <Input
          bind:value={newRootPath}
          aria-label="Folder path"
          aria-describedby="audit-root-hint"
          placeholder={'D:\\production'}
        />
        <Button variant="outline" onclick={onAddRoot} disabled={scanning}>Add folder</Button>
      </div>
      <p id="audit-root-hint" class="text-xs text-muted-foreground">
        Leave the field empty and Add folder opens a folder picker.
      </p>

      <div class="space-y-2 text-sm">
        <Label class="items-start gap-2 font-normal">
          <Checkbox
            checked={measureSizes}
            onCheckedChange={(v) => {
              measureSizes = v === true;
              onPersist();
            }}
            disabled={scanning}
          />
          <span>
            Measure reclaimable size
            <span class="text-xs text-muted-foreground">walks build output — slower</span>
          </span>
        </Label>
        <Label class="items-start gap-2 font-normal">
          <Checkbox
            checked={markMissing}
            onCheckedChange={(v) => {
              markMissing = v === true;
              onPersist();
            }}
            disabled={scanning}
          />
          <span>
            Archive projects no longer on disk
            <span class="text-xs text-muted-foreground">
              roots that failed to open are skipped
            </span>
          </span>
        </Label>
      </div>
    </div>
  {/if}

  {#if scanning && progress}
    <div class="space-y-1">
      <Progress value={percent} aria-label="Scan progress" />
      <!-- The path changes many times a second; announcing it would drown
           out everything else, so the pollable progressbar carries it. -->
      <p class="truncate font-mono text-xs text-muted-foreground" aria-hidden="true">
        {progress.phase === 'find' ? 'walking' : `${progress.done}/${progress.total}`}
        {progress.phase === 'size' ? ' sizing' : ''} — {progress.path}
      </p>
    </div>
  {/if}

  {#if error}
    <p class="text-sm text-destructive" role="alert">{error}</p>
  {/if}
  {#if importSummary}
    <p class="text-xs text-muted-foreground" role="status">{importSummary}</p>
  {/if}
</section>
