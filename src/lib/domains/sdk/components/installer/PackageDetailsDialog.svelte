<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Badge } from '$lib/components/ui/badge';
  import Loader2 from '~icons/fa6-solid/spinner';
  import type { PackageDetails } from '$lib/domains/sdk/services/packageManagerService';

  interface Props {
    open: boolean;
    loading: boolean;
    details: PackageDetails | null;
  }
  let { open = $bindable(), loading, details }: Props = $props();

  const fields = $derived(
    details
      ? [
          { label: 'Version', value: details.version },
          { label: 'Publisher', value: details.publisher },
          { label: 'Description', value: details.description },
        ]
      : []
  );
</script>

<Dialog bind:open>
  <DialogContent class="max-h-[80vh] max-w-2xl overflow-y-auto">
    <DialogHeader>
      <DialogTitle>{details?.name || 'Package Details'}</DialogTitle>
      <DialogDescription>
        Package information from {details?.source || 'package manager'}
      </DialogDescription>
    </DialogHeader>
    {#if loading}
      <div class="flex items-center justify-center py-8">
        <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    {:else if details}
      <div class="space-y-4">
        <div>
          <p class="text-sm font-medium">ID</p>
          <p class="font-mono text-sm text-muted-foreground">{details.id}</p>
        </div>
        {#each fields as f}
          {#if f.value}
            <div>
              <p class="text-sm font-medium">{f.label}</p>
              <p class="text-sm text-muted-foreground">{f.value}</p>
            </div>
          {/if}
        {/each}
        {#if details.homepage}
          <div>
            <p class="text-sm font-medium">Homepage</p>
            <a
              href={details.homepage}
              target="_blank"
              class="text-sm text-blue-600 hover:underline"
            >
              {details.homepage}
            </a>
          </div>
        {/if}
        {#if details.license}
          <div>
            <p class="text-sm font-medium">License</p>
            <p class="text-sm text-muted-foreground">{details.license}</p>
          </div>
        {/if}
        {#if details.dependencies.length > 0}
          <div>
            <p class="text-sm font-medium">Dependencies</p>
            <div class="mt-2 flex flex-wrap gap-2">
              {#each details.dependencies as dep}
                <Badge variant="secondary" class="text-xs">{dep}</Badge>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </DialogContent>
</Dialog>
