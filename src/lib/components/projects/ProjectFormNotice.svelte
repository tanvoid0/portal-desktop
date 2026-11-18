<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';

  interface Props {
    kind: 'success' | 'error';
    message: string;
  }
  let { kind, message }: Props = $props();
</script>

{#if kind === 'success'}
  <Card class="mb-6 border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
    <CardContent class="pt-6">
      <div class="flex items-center gap-2 text-green-800 dark:text-green-200">
        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
        <span class="font-medium">Success</span>
      </div>
      <p class="mt-1 text-sm text-green-600 dark:text-green-300">{message}</p>
    </CardContent>
  </Card>
{:else}
  <Card class="mb-6 border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950">
    <CardContent class="pt-6">
      <div class="flex items-center gap-2 text-red-800 dark:text-red-200">
        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <span class="font-medium">Error</span>
      </div>
      <p class="mt-1 text-sm text-red-600 dark:text-red-300">{message}</p>
    </CardContent>
  </Card>
{/if}
