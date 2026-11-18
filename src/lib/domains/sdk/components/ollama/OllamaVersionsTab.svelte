<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import OllamaPanelState from './OllamaPanelState.svelte';
  import OllamaRefreshIcon from './OllamaRefreshIcon.svelte';

  interface Props {
    versions: any[];
    loading: boolean;
    error: string | null;
    onload: () => void;
    oninstall: (version: string) => void;
  }
  let { versions, loading, error, onload, oninstall }: Props = $props();
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <div>
        <CardTitle>Ollama Versions</CardTitle>
        <CardDescription>Install and manage Ollama versions</CardDescription>
      </div>
      <Button variant="outline" onclick={onload} disabled={loading}>
        <OllamaRefreshIcon />
        Refresh
      </Button>
    </div>
  </CardHeader>
  <CardContent>
    <OllamaPanelState {loading} {error} loadingText="Loading versions..." onretry={onload}>
      {#if versions.length > 0}
        <div class="grid gap-4">
          {#each versions as version}
            <div class="flex items-center justify-between rounded-lg border p-4">
              <div class="flex items-center gap-4">
                <div class="flex items-center gap-2">
                  <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                    />
                  </svg>
                  <span class="font-medium">Ollama-{version.version}</span>
                </div>
                <div class="text-sm text-muted-foreground">Version {version.version}</div>
                {#if version.size && version.size !== '0.0 MB'}
                  <div class="text-sm text-muted-foreground">{version.size}</div>
                {/if}
              </div>
              <div class="flex items-center gap-2">
                {#if version.installed}
                  <Badge variant="default" class="bg-green-100 text-green-800">
                    <svg class="mr-1 h-3 w-3" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M5 13l4 4L19 7" />
                    </svg>
                    Installed
                  </Badge>
                  {#if version.active}
                    <Badge variant="secondary" class="bg-blue-100 text-blue-800">Active</Badge>
                  {/if}
                {:else}
                  <Button size="sm" onclick={() => oninstall(version.version)}>
                    <svg class="mr-1 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                      />
                    </svg>
                    Install
                  </Button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="p-8 text-center">
          <p class="text-muted-foreground">No versions available.</p>
          <Button onclick={onload} class="mt-4">Load Versions</Button>
        </div>
      {/if}
    </OllamaPanelState>
  </CardContent>
</Card>
