<script lang="ts">
  import LoadingSpinner from '$lib/components/ui/loading-spinner.svelte';
  import { Progress } from '$lib/components/ui/progress';
  interface Props {
    title: string;
    status: string;
    progress?: number;
    showSpinner?: boolean;
  }

  let { title, status, progress = 0, showSpinner = true }: Props = $props();
</script>

<div class="rounded-lg border bg-blue-50 p-4">
  <div class="flex items-center gap-4">
    {#if showSpinner}
      <LoadingSpinner class="text-blue-600" />
    {/if}
    <div class="flex-1">
      <div class="font-medium">{title}</div>
      <div class="text-sm text-muted-foreground">{status}</div>
      <Progress value={progress} class="mt-2 bg-gray-200" indicatorClass="bg-blue-600" />
    </div>
  </div>
</div>
