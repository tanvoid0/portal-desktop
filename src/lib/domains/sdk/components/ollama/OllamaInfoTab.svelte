<!-- Static placeholder tabs (Configuration, Log): no backend command for either yet. -->
<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';

  let { kind }: { kind: 'configuration' | 'log' } = $props();
</script>

<Card>
  {#if kind === 'configuration'}
    <CardHeader>
      <CardTitle>Configuration</CardTitle>
      <CardDescription>Configure Ollama settings</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <p class="text-sm text-muted-foreground">
        Reading and editing the Ollama config file is not available yet.
      </p>
      <div class="text-sm text-muted-foreground">
        <p class="mb-2 font-medium">Available Settings:</p>
        <ul class="list-inside list-disc space-y-1 text-xs">
          <li><code>OLLAMA_HOST</code> - Host and port (default: 0.0.0.0:11434)</li>
          <li><code>OLLAMA_KEEP_ALIVE</code> - Keep models in memory duration (default: 5m)</li>
          <li><code>OLLAMA_DEBUG</code> - Enable debug logging</li>
          <li><code>OLLAMA_MAX_LOADED_MODELS</code> - Maximum number of loaded models</li>
          <li><code>OLLAMA_NUM_PARALLEL</code> - Number of parallel requests</li>
        </ul>
      </div>
    </CardContent>
  {:else}
    <CardHeader>
      <CardTitle>Service Logs</CardTitle>
      <CardDescription>View Ollama service logs</CardDescription>
    </CardHeader>
    <CardContent>
      <p class="text-sm text-muted-foreground">Log viewing is not available yet.</p>
    </CardContent>
  {/if}
</Card>
