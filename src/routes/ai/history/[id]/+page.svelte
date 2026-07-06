<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  onMount(() => {
    const conversationId = $page.params.id;
    if (conversationId) {
      goto(`/ai/chat?id=${conversationId}`);
    } else {
      goto("/ai/chat");
    }
  });
</script>

<div class="flex h-full w-full items-center justify-center p-6">
  <p class="text-muted-foreground">Redirecting to chat...</p>
</div>
