<!-- Centered spinner for a panel that is loading. -->
<script lang="ts">
  import Loader2 from '~icons/fa6-solid/spinner';

  let { class: className = 'py-12' }: { class?: string } = $props();
</script>

<div class="flex items-center justify-center {className}">
  <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
</div>
