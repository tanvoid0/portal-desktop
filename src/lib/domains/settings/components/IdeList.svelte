<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import Laptop from '~icons/fa6-solid/laptop';
  import Plus from '~icons/fa6-solid/plus';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Code2 from '~icons/fa6-solid/code';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { logger } from '$lib/domains/shared';
  import { ideService, type IdeConfig } from '$lib/domains/ide';
  import IdeEditDialog from './IdeEditDialog.svelte';

  interface Props {
    ides: IdeConfig[];
    isLoading?: boolean;
    onRefresh?: () => void;
    /** After an IDE was added, edited, deleted or made default. */
    onChanged?: () => void;
    /** After an IDE was deleted (its framework mappings may have gone with it). */
    onDeleted?: () => void;
  }

  let {
    ides,
    isLoading = false,
    onRefresh = () => {},
    onChanged = () => {},
    onDeleted = () => {},
  }: Props = $props();

  /** Strip the Windows extended-length path prefix (\\?\). */
  function normalizePath(path: string): string {
    return path.startsWith('\\\\?\\') ? path.slice(4) : path;
  }

  // The old per-editor checks all collapse to this ("nvim" contains "vim").
  const CODE_ICON =
    /code|idea|pycharm|webstorm|clion|goland|phpstorm|rubymine|rider|studio|sublime|vim|emacs/;

  function getIdeIcon(ideName: string, executable: string) {
    return CODE_ICON.test(ideName.toLowerCase()) || CODE_ICON.test(executable.toLowerCase())
      ? Code2
      : Laptop;
  }

  let isDetecting = $state(false);
  let detectedIdes = $state<string[]>([]);
  let dialogOpen = $state(false);
  let editingIde = $state<IdeConfig | null>(null);
  let initialName = $state('');
  let initialExecutable = $state('');

  function openDialog(ide: IdeConfig | null, name: string, executable: string) {
    editingIde = ide;
    initialName = name;
    initialExecutable = executable;
    dialogOpen = true;
  }

  async function detectIdes() {
    try {
      isDetecting = true;
      detectedIdes = await ideService.detectInstalledIdes();
      logger.info('IDEs detected', { context: 'IdeSettings', count: detectedIdes.length });

      if (detectedIdes.length === 0) {
        toast.info('No IDEs detected automatically');
      }
    } catch (error) {
      toast.error('Failed to detect IDEs', error);
    } finally {
      isDetecting = false;
    }
  }

  async function deleteIde(ide: IdeConfig) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${ide.name}"?`,
      'Delete IDE'
    );
    if (!confirmed) return;

    try {
      if (ide.id) {
        await ideService.deleteIde(ide.id);
        toast.success('IDE deleted successfully');
        onChanged();
        onDeleted();
      }
    } catch (error: any) {
      toast.error('Failed to delete IDE', error);
    }
  }

  async function setDefaultIde(ide: IdeConfig) {
    try {
      if (ide.id) {
        await ideService.setDefaultIde(ide.id);
        toast.success('Default IDE updated');
        onChanged();
      }
    } catch (error: any) {
      toast.error('Failed to set default IDE', error);
    }
  }
</script>

<div class="flex items-center justify-between">
  <div>
    <h2 class="flex items-center gap-2 text-xl font-bold tracking-tight">
      <Code2 class="h-5 w-5" />
      IDE Configuration
    </h2>
    <p class="text-sm text-muted-foreground">
      Configure integrated development environments for project opening
    </p>
  </div>
  <div class="flex gap-2">
    <Button variant="outline" size="sm" onclick={detectIdes} disabled={isDetecting}>
      {#if isDetecting}
        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
      {:else}
        <Search class="mr-2 h-4 w-4" />
      {/if}
      Detect IDEs
    </Button>
    <Button variant="outline" size="sm" onclick={onRefresh} disabled={isLoading}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
    <Button variant="outline" size="sm" onclick={() => openDialog(null, '', '')}>
      <Plus class="mr-2 h-4 w-4" />
      Add IDE
    </Button>
  </div>
</div>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Code2 class="h-5 w-5" />
      Configured IDEs
      <Badge variant="outline">{ides.length}</Badge>
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if isLoading}
      <div class="flex items-center justify-center py-8">
        <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
        <span class="ml-2 text-muted-foreground">Loading IDEs...</span>
      </div>
    {:else if ides.length === 0}
      <div class="py-8 text-center">
        <Code2 class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
        <p class="mb-4 text-muted-foreground">No IDEs configured</p>
        <div class="flex justify-center gap-2">
          <Button onclick={detectIdes} disabled={isDetecting}>
            {#if isDetecting}
              <Loader2 class="mr-2 h-4 w-4 animate-spin" />
            {:else}
              <Search class="mr-2 h-4 w-4" />
            {/if}
            Detect IDEs
          </Button>
          <Button onclick={() => openDialog(null, '', '')} variant="outline">
            <Plus class="mr-2 h-4 w-4" />
            Add Manually
          </Button>
        </div>
      </div>
    {:else}
      <div class="space-y-3">
        {#each ides as ide}
          {@const IdeIcon = getIdeIcon(ide.name, ide.executable)}
          <div
            class="flex items-center justify-between rounded-md border p-4 transition-colors hover:bg-accent"
          >
            <div class="flex items-center gap-4">
              <IdeIcon class="h-5 w-5 text-muted-foreground" />
              <div class="flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium">{ide.name}</span>
                  {#if ide.is_default}
                    <Badge variant="default" class="text-xs">Default</Badge>
                  {/if}
                </div>
                <p class="truncate text-sm text-muted-foreground">
                  {normalizePath(ide.executable)}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-2">
              {#if !ide.is_default}
                <Button variant="ghost" size="sm" onclick={() => setDefaultIde(ide)}>
                  Set Default
                </Button>
              {/if}
              <Button
                variant="ghost"
                size="sm"
                aria-label="Edit IDE"
                onclick={() => openDialog(ide, ide.name, normalizePath(ide.executable))}
              >
                <Edit class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                aria-label="Delete IDE"
                onclick={() => deleteIde(ide)}
                class="text-destructive hover:text-destructive"
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    {#if detectedIdes.length > 0}
      <Separator class="my-4" />
      <div>
        <p class="mb-2 text-sm font-medium">Detected IDEs</p>
        <div class="flex flex-wrap gap-2">
          {#each detectedIdes as executablePath}
            {@const executableName = executablePath.split(/[/\\]/).pop() || executablePath}
            {@const normalizedPath = normalizePath(executablePath)}
            {@const alreadyAdded = ides.some(
              (ide) => normalizePath(ide.executable) === normalizedPath
            )}
            {#if !alreadyAdded}
              <Button
                variant="outline"
                size="sm"
                onclick={() =>
                  openDialog(
                    null,
                    executableName.replace(/\.exe$/i, '').replace(/\.app$/i, ''),
                    normalizedPath
                  )}
              >
                <Plus class="mr-1 h-3 w-3" />
                {executableName}
              </Button>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </CardContent>
</Card>

<IdeEditDialog
  bind:open={dialogOpen}
  ide={editingIde}
  {initialName}
  {initialExecutable}
  onSaved={onChanged}
/>
