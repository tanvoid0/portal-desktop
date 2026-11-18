<!-- Manager selector + status badges + package search box for the software installer. -->
<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Loader2 from '~icons/fa6-solid/spinner';
  import type { PackageManagerInfo } from '$lib/domains/sdk/services/packageManagerService';

  interface Props {
    managers: string[];
    selected: string;
    info: PackageManagerInfo | null;
    loading: boolean;
    searching: boolean;
    query: string;
    onselect: (manager: string) => void;
    onsearchinput: () => void;
  }
  let {
    managers,
    selected,
    info,
    loading,
    searching,
    query = $bindable(),
    onselect,
    onsearchinput,
  }: Props = $props();

  const DISPLAY_NAMES: Record<string, string> = {
    winget: 'Windows Package Manager',
    scoop: 'Scoop',
    chocolatey: 'Chocolatey',
    cargo: 'Cargo',
    homebrew: 'Homebrew',
    npm: 'NPM',
    pip: 'Pip',
  };
</script>

<Card>
  <CardContent class="pt-6">
    <div class="flex items-center gap-4">
      <div class="flex-shrink-0" style="width: 250px;">
        <div class="mb-2 text-sm font-medium">Package Manager</div>
        <Select
          options={managers.map((m) => ({ value: m, label: DISPLAY_NAMES[m] || m }))}
          value={selected}
          placeholder="Select a package manager"
          onSelect={onselect}
          disabled={loading || managers.length === 0}
        />
        {#if info}
          <div class="mt-2 flex flex-wrap items-center gap-2">
            {#if info.available}
              <Badge variant="default" class="bg-green-100 text-xs text-green-800">
                <CheckCircle class="mr-1 h-3 w-3" />
                Available
              </Badge>
            {:else}
              <Badge variant="outline" class="text-xs text-gray-500">
                <XCircle class="mr-1 h-3 w-3" />
                Not Available
              </Badge>
            {/if}
            {#if info.requires_elevation}
              <Badge variant="secondary" class="text-xs">Requires Admin</Badge>
            {/if}
          </div>
        {/if}
      </div>

      {#if selected && info?.available}
        <div class="flex-1">
          <div class="mb-2 text-sm font-medium">Search Packages</div>
          <div class="flex items-center gap-2">
            <div class="relative flex-1">
              <Search
                class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
              />
              <Input
                type="text"
                placeholder="Search packages..."
                bind:value={query}
                oninput={onsearchinput}
                class="pl-10"
                disabled={!info.supports_search}
                aria-label="Search packages"
              />
            </div>
            {#if searching}
              <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
            {/if}
          </div>
        </div>
      {/if}
    </div>
    {#if info}
      <div class="mt-3 text-xs text-muted-foreground">
        Version: {info.version} | Platform: {info.platform}
      </div>
    {/if}
  </CardContent>
</Card>
