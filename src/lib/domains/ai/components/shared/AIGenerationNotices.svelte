<!-- Info box plus the "not configured" / "model not installed" hints shown under the generation input. -->
<script lang="ts">
  import FaCircleInfo from '~icons/fa6-solid/circle-info';
  import FaTriangleExclamation from '~icons/fa6-solid/triangle-exclamation';
  import FaGear from '~icons/fa6-solid/gear';
  import FaBoxOpen from '~icons/fa6-solid/box-open';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    infoContent?: string | { title: string; items: string[] };
    error: string | null;
    onOpenSettings: () => void;
  }

  let { infoContent, error, onOpenSettings }: Props = $props();
</script>

{#if infoContent}
  <div class="rounded-lg border bg-muted/50 p-4">
    <div class="flex gap-2">
      <FaCircleInfo class="mt-0.5 h-5 w-5 shrink-0 text-muted-foreground" />
      <div class="space-y-1 text-sm text-muted-foreground">
        {#if typeof infoContent === 'string'}
          <p>{infoContent}</p>
        {:else}
          <p class="font-medium">{infoContent.title}</p>
          <ul class="ml-2 list-inside list-disc space-y-0.5">
            {#each infoContent.items as item}
              <li>{item}</li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if error && error.includes('not configured')}
  <div
    class="rounded-lg border border-amber-200 bg-amber-50 p-4 dark:border-amber-800 dark:bg-amber-900/20"
  >
    <div class="flex gap-2">
      <FaTriangleExclamation class="mt-0.5 h-5 w-5 shrink-0 text-amber-600 dark:text-amber-400" />
      <div class="flex-1 space-y-2 text-sm">
        <p class="font-medium text-amber-900 dark:text-amber-100">AI Provider Not Configured</p>
        <p class="text-amber-800 dark:text-amber-200">
          To use AI features, configure Agent Platform under Settings → AI Providers.
        </p>
        <Button
          variant="outline"
          size="sm"
          onclick={onOpenSettings}
          class="mt-2 border-amber-300 text-amber-900 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-100 dark:hover:bg-amber-900/40"
        >
          <FaGear class="mr-2 h-4 w-4" />
          Configure AI Provider
        </Button>
      </div>
    </div>
  </div>
{/if}

{#if error && (error.includes('not installed') || (error.includes('model') && error.includes('not found')))}
  <div
    class="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-900/20"
  >
    <div class="flex gap-2">
      <FaBoxOpen class="mt-0.5 h-5 w-5 shrink-0 text-blue-600 dark:text-blue-400" />
      <div class="flex-1 space-y-2 text-sm">
        <p class="font-medium text-blue-900 dark:text-blue-100">Model Not Installed</p>
        <p class="text-blue-800 dark:text-amber-200">{error}</p>
        <p class="text-xs text-blue-700 dark:text-blue-300">
          You can install the model in the terminal or use the "Load Models" button in Settings to
          see available models.
        </p>
        <Button
          variant="outline"
          size="sm"
          onclick={onOpenSettings}
          class="mt-2 border-blue-300 text-blue-900 hover:bg-blue-100 dark:border-blue-700 dark:text-blue-100 dark:hover:bg-blue-900/40"
        >
          <FaGear class="mr-2 h-4 w-4" />
          Open Settings
        </Button>
      </div>
    </div>
  </div>
{/if}
