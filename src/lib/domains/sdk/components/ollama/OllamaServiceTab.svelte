<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import OllamaPanelState from './OllamaPanelState.svelte';

  interface Props {
    serviceInfo: any;
    loading: boolean;
    error: string | null;
    onretry: () => void;
    onstart: () => void;
    onstop: () => void;
    onrestart: () => void;
    onviewlogs: () => void;
    oncheckupdates: () => void;
    onupdate: () => void;
  }
  let {
    serviceInfo,
    loading,
    error,
    onretry,
    onstart,
    onstop,
    onrestart,
    onviewlogs,
    oncheckupdates,
    onupdate,
  }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle>Ollama Service</CardTitle>
    <CardDescription>Manage your Ollama service instance</CardDescription>
  </CardHeader>
  <CardContent>
    <OllamaPanelState {loading} {error} loadingText="Loading service information..." {onretry}>
      <div class="space-y-4">
        <div>
          <h4 class="mb-2 font-medium">Service Status</h4>
          <p class="text-sm text-muted-foreground">
            {#if serviceInfo?.running}
              Service is running on port {serviceInfo.port || '11434'}
            {:else}
              Service is {serviceInfo?.status || 'not running'}
            {/if}
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          {#if serviceInfo?.running}
            <Button variant="destructive" onclick={onstop}>Stop Service</Button>
          {:else}
            <Button onclick={onstart}>Start Service</Button>
          {/if}
          <Button variant="outline" onclick={onrestart}>Restart Service</Button>
          <Button variant="outline" onclick={onviewlogs}>View Logs</Button>
          <Button variant="outline" onclick={oncheckupdates}>Check Updates</Button>
          <Button variant="outline" onclick={onupdate}>Update Ollama</Button>
        </div>
      </div>
    </OllamaPanelState>
  </CardContent>
</Card>
