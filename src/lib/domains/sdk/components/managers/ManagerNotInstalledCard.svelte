<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import type { SDKManager } from '../../utils/sdkSidebarItems';

  let { manager }: { manager: SDKManager } = $props();
</script>

<Card>
  <CardContent class="pt-6">
    <p class="mb-4 text-muted-foreground">
      This SDK manager is not installed. Install it to manage SDK versions.
    </p>
    {#if manager.install_command}
      <div class="space-y-2">
        <p class="text-sm font-medium">Installation Command:</p>
        <code class="block rounded bg-muted p-2 font-mono text-xs break-all">
          {manager.install_command}
        </code>
      </div>
    {/if}
    {#if manager.website}
      <div class="mt-4">
        <Button variant="outline" onclick={() => window.open(manager.website!, '_blank')}>
          <ExternalLink class="mr-2 h-4 w-4" />
          Visit Website
        </Button>
      </div>
    {/if}
  </CardContent>
</Card>
