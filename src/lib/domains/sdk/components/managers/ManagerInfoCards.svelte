<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import type { SDKManager } from '../../utils/sdkSidebarItems';

  let { manager }: { manager: SDKManager } = $props();
</script>

<div class="grid gap-6 md:grid-cols-2">
  <Card>
    <CardHeader>
      <CardTitle>Manager Information</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <div>
        <p class="text-sm font-medium">ID</p>
        <p class="font-mono text-sm text-muted-foreground">{manager.id}</p>
      </div>
      {#if manager.version}
        <div>
          <p class="text-sm font-medium">Manager Version</p>
          <p class="text-sm text-muted-foreground">{manager.version}</p>
        </div>
      {/if}
      {#if manager.website}
        <div>
          <p class="text-sm font-medium">Website</p>
          <a
            href={manager.website}
            target="_blank"
            class="flex items-center gap-1 text-sm text-blue-600 hover:underline"
          >
            {manager.website}
            <ExternalLink class="h-3 w-3" />
          </a>
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Capabilities</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="flex flex-wrap gap-2">
        {#if manager.supports_installation}
          <Badge variant="secondary" class="text-xs">Installation</Badge>
        {/if}
        {#if manager.supports_version_switching}
          <Badge variant="secondary" class="text-xs">Version Switching</Badge>
        {/if}
      </div>
    </CardContent>
  </Card>
</div>
