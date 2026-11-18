<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Textarea } from '$lib/components/ui/textarea';
  import Save from '~icons/fa6-solid/floppy-disk';
  import RotateCcw from '~icons/fa6-solid/rotate-left';
  import Download from '~icons/fa6-solid/download';
  import FileText from '~icons/fa6-solid/file-lines';
  import type { ConfigFile } from './serviceConfigTypes';

  interface Props {
    configFiles: ConfigFile[];
    selectedFile: ConfigFile | null;
    editedContent: string;
    hasChanges: boolean;
    loading: boolean;
    onselect: (file: ConfigFile) => void;
    onchange: () => void;
    onsave: () => void;
    onreset: () => void;
    ondownload: () => void;
  }

  let {
    configFiles,
    selectedFile,
    editedContent = $bindable(),
    hasChanges,
    loading,
    onselect,
    onchange,
    onsave,
    onreset,
    ondownload,
  }: Props = $props();

  const FILE_ICONS: Record<string, string> = { json: '{}', yaml: 'Y', ini: 'I', xml: 'X' };
  const fileIcon = (format: string) => FILE_ICONS[format] ?? 'T';
</script>

<div class="w-1/3 space-y-2">
  <h3 class="mb-2 font-medium">Configuration Files</h3>
  <div class="space-y-1">
    {#each configFiles as file}
      <Button
        variant={selectedFile?.name === file.name ? 'default' : 'ghost'}
        class="w-full justify-start"
        onclick={() => onselect(file)}
      >
        <span class="mr-2">{fileIcon(file.format)}</span>
        {file.name}
      </Button>
    {/each}
  </div>
</div>

<div class="flex flex-1 flex-col">
  {#if selectedFile}
    <div class="mb-4 flex items-center justify-between">
      <div>
        <h3 class="font-medium">{selectedFile.name}</h3>
        <p class="text-sm text-muted-foreground">{selectedFile.path}</p>
      </div>
      <div class="flex items-center gap-2">
        <Badge variant="outline">{selectedFile.format}</Badge>
        <Button variant="outline" size="sm" onclick={ondownload}>
          <Download class="mr-2 h-4 w-4" />
          Download
        </Button>
      </div>
    </div>

    <Textarea
      bind:value={editedContent}
      oninput={onchange}
      class="flex-1 font-mono text-sm"
      placeholder="Configuration content..."
    />

    <div class="mt-4 flex items-center justify-between">
      <div class="flex items-center gap-2">
        {#if hasChanges}
          <Badge variant="secondary">Unsaved changes</Badge>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <Button variant="outline" onclick={onreset} disabled={!hasChanges}>
          <RotateCcw class="mr-2 h-4 w-4" />
          Reset
        </Button>
        <Button onclick={onsave} disabled={!hasChanges || loading}>
          <Save class="mr-2 h-4 w-4" />
          Save
        </Button>
      </div>
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center text-muted-foreground">
      <div class="text-center">
        <FileText class="mx-auto mb-4 h-12 w-12 opacity-50" />
        <p>Select a configuration file to edit</p>
      </div>
    </div>
  {/if}
</div>
