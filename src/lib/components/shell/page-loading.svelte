<script lang="ts">
  import LoadingSpinner from "$lib/components/ui/loading-spinner.svelte";

  interface Props {
    message?: string;
    class?: string;
  }

  let { message = "Loading...", class: className }: Props = $props();
</script>

<div class="flex items-center justify-center py-12 {className ?? ''}">
  <LoadingSpinner size="lg" text={message} />
</div>
