<script lang="ts">
  import ListIcon from '~icons/fa6-solid/list';
  import GridIcon from '~icons/fa6-solid/table-cells-large';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
  import Select from '$lib/components/ui/select.svelte';
  import { PageFilters } from '$lib/components/shell';
  import { PROJECT_STAGES } from '$lib/domains/projects/types';
  import type { ViewMode } from '$lib/domains/projects/utils/viewPrefs';

  interface Props {
    searchQuery: string;
    filterStack: string | null;
    filterActivity: string | null;
    filterStage: string | null;
    starredOnly: boolean;
    showArchived: boolean;
    sortBy: 'recent' | 'name';
    viewMode: ViewMode;
    uniqueStacks: string[];
    isFiltered: boolean;
    onClear: () => void;
    onViewModeChange: (mode: string) => void;
  }
  let {
    searchQuery = $bindable(),
    filterStack = $bindable(),
    filterActivity = $bindable(),
    filterStage = $bindable(),
    starredOnly = $bindable(),
    showArchived = $bindable(),
    sortBy = $bindable(),
    viewMode,
    uniqueStacks,
    isFiltered,
    onClear,
    onViewModeChange,
  }: Props = $props();
</script>

<PageFilters bind:searchQuery searchPlaceholder="Search projects...">
  {#snippet filters()}
    <Select
      value={filterStack ?? ''}
      placeholder="All stacks"
      options={[
        { value: '', label: 'All stacks' },
        ...uniqueStacks.map((n) => ({ value: n, label: n })),
      ]}
      onSelect={(value) => (filterStack = value || null)}
      class="w-[170px]"
    />

    <Select
      value={filterActivity ?? ''}
      placeholder="Any activity"
      options={[
        { value: '', label: 'Any activity' },
        { value: 'active', label: 'Active' },
        { value: 'recent', label: 'Recent' },
        { value: 'dormant', label: 'Dormant' },
        { value: 'stale', label: 'Stale' },
      ]}
      onSelect={(value) => (filterActivity = value || null)}
      class="w-[150px]"
    />

    <Select
      value={filterStage ?? ''}
      placeholder="Any stage"
      options={[
        { value: '', label: 'Any stage' },
        ...PROJECT_STAGES.map((s) => ({ value: s.value, label: s.label })),
        { value: 'none', label: 'Not set' },
      ]}
      onSelect={(value) => (filterStage = value || null)}
      class="w-[150px]"
    />

    <Select
      value={sortBy}
      options={[
        { value: 'recent', label: 'Recently active' },
        { value: 'name', label: 'Name' },
      ]}
      onSelect={(value) => (sortBy = value as typeof sortBy)}
      class="w-[170px]"
    />

    <div class="flex items-center gap-2">
      <Switch id="starred-only" bind:checked={starredOnly} />
      <Label for="starred-only" class="text-sm">Starred only</Label>
    </div>

    <div class="flex items-center gap-2">
      <Switch id="show-archived" bind:checked={showArchived} />
      <Label for="show-archived" class="text-sm">Show archived</Label>
    </div>
  {/snippet}

  {#snippet actions()}
    {#if isFiltered}
      <Button variant="ghost" size="sm" onclick={onClear} class="text-muted-foreground">
        Clear filters
      </Button>
    {/if}

    <ToggleGroup type="single" value={viewMode} onValueChange={onViewModeChange} variant="outline">
      <ToggleGroupItem value="list" aria-label="List view">
        <ListIcon class="h-3.5 w-3.5" />
      </ToggleGroupItem>
      <ToggleGroupItem value="grid" aria-label="Grid view">
        <GridIcon class="h-3.5 w-3.5" />
      </ToggleGroupItem>
    </ToggleGroup>
  {/snippet}
</PageFilters>
