<script lang="ts">
  export let title: string;
  export let status: string;
  export let progress: number = 0;
  export let showSpinner: boolean = true;
</script>

<div class="rounded-lg border bg-blue-50 p-4">
  <div class="flex items-center gap-4">
    {#if showSpinner}
      <div
        class="h-6 w-6 animate-spin rounded-full border-b-2 border-blue-600"
      ></div>
    {/if}
    <div class="flex-1">
      <div class="font-medium">{title}</div>
      <div class="text-sm text-muted-foreground">{status}</div>
      <div class="mt-2 h-2 w-full rounded-full bg-gray-200">
        <div
          class="h-2 rounded-full bg-blue-600 transition-all duration-300"
          style="width: {Math.max(0, Math.min(100, progress))}%"
        ></div>
      </div>
    </div>
  </div>
</div>
