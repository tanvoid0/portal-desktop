<script lang="ts">
  import { AlertCircle, RotateCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";

  interface Props {
    title?: string;
    message: string;
    onRetry?: () => void;
  }

  let {
    title = "Something went wrong",
    message,
    onRetry,
  }: Props = $props();
</script>

<Card>
  <CardContent class="flex flex-col items-center justify-center py-12">
    <AlertCircle class="mb-4 h-12 w-12 text-destructive" />
    <h3 class="mb-2 text-lg font-semibold text-destructive">{title}</h3>
    <p class="mb-4 max-w-md text-center text-muted-foreground">{message}</p>
    {#if onRetry}
      <Button variant="outline" onclick={onRetry}>
        <RotateCw class="mr-2 h-4 w-4" />
        Retry
      </Button>
    {/if}
  </CardContent>
</Card>
