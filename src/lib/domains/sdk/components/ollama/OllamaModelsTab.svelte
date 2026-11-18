<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import ModelList from '$lib/components/ModelList.svelte';
  import ModelTreeList from '$lib/components/ModelTreeList.svelte';
  import type { OllamaModels } from '../../state/ollamaModels.svelte';
  import OllamaRefreshIcon from './OllamaRefreshIcon.svelte';

  interface Props {
    state: OllamaModels;
    serviceRunning: boolean;
    onstart: () => void;
  }
  let { state: m, serviceRunning, onstart }: Props = $props();
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <div>
          <CardTitle>Models</CardTitle>
          <CardDescription>Manage your AI models</CardDescription>
        </div>
        <div class="flex items-center gap-2">
          <Button
            variant={m.tab === 'local' ? 'default' : 'outline'}
            size="sm"
            onclick={() => m.show('local')}
          >
            Local
          </Button>
          <Button
            variant={m.tab === 'library' ? 'default' : 'outline'}
            size="sm"
            onclick={() => m.show('library')}
          >
            Library
          </Button>
        </div>
      </div>
      <Button variant="outline" onclick={() => m.reload()} disabled={m.loading}>
        <OllamaRefreshIcon />
        Refresh
      </Button>
    </div>
  </CardHeader>
  <CardContent>
    {#if m.tab === 'local'}
      {#if !serviceRunning}
        <div class="p-8 text-center">
          <div class="flex flex-col items-center gap-4">
            <svg
              class="h-12 w-12 text-muted-foreground"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
              />
            </svg>
            <div>
              <h3 class="text-lg font-semibold">Ollama Service Not Running</h3>
              <p class="text-muted-foreground">
                Please start the Ollama service to view installed models.
              </p>
            </div>
            <Button onclick={onstart} class="mt-2">
              <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1m-6-8h8a2 2 0 012 2v8a2 2 0 01-2 2H8a2 2 0 01-2-2V8a2 2 0 012-2z"
                />
              </svg>
              Start Service
            </Button>
          </div>
        </div>
      {:else}
        <ModelList
          models={m.models}
          isInstalled={true}
          error={m.error}
          installingModel={m.installingModel}
          installationProgress={m.installationProgress}
          installationStatus={m.installationStatus}
          onInstall={(name: string) => m.install(name)}
          onRemove={(name: string) => m.remove(name)}
          onRetry={() => m.loadLocal()}
          onBrowseAvailable={() => m.show('library')}
        />
      {/if}
    {:else}
      <ModelTreeList
        models={m.availableModels}
        error={m.error}
        installingModel={m.installingModel}
        installationProgress={m.installationProgress}
        installationStatus={m.installationStatus}
        onInstall={(name: string) => m.install(name)}
        onRetry={() => m.loadLibrary()}
        onCancel={() => m.cancelInstall()}
      />
    {/if}
  </CardContent>
</Card>
