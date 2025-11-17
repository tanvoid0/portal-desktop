<!-- PodsFilters - Search and filter pods -->
<script lang="ts">
  import { Input } from '@/lib/components/ui/input';
  import { Button } from '@/lib/components/ui/button';
  import Select from '@/lib/components/ui/select.svelte';
  import { ResourceStatus } from '../../core/types';
  
  interface Props {
    searchQuery: string;
    statusFilter: string;
    onSearchChange: (query: string) => void;
    onStatusFilterChange: (status: string) => void;
    onClear: () => void;
  }
  
  let {
    searchQuery,
    statusFilter,
    onSearchChange,
    onStatusFilterChange,
    onClear
  }: Props = $props();
  
  const statusOptions = [
    { value: '', label: 'All Statuses' },
    { value: ResourceStatus.RUNNING, label: 'Running' },
    { value: ResourceStatus.PENDING, label: 'Pending' },
    { value: ResourceStatus.FAILED, label: 'Failed' },
    { value: ResourceStatus.SUCCEEDED, label: 'Succeeded' }
  ];
  
  const hasFilters = $derived(searchQuery || statusFilter);
</script>

<div class="flex flex-col sm:flex-row gap-4 items-start sm:items-center">
  <div class="flex-1 min-w-0">
    <Input
      type="text"
      placeholder="Search pods by name..."
      value={searchQuery}
      oninput={(e) => onSearchChange(e.currentTarget.value)}
      class="w-full"
    />
  </div>
  
  <Select
    options={statusOptions}
    bind:value={statusFilter}
    onSelect={onStatusFilterChange}
    placeholder="All Statuses"
  />
  
  {#if hasFilters}
    <Button variant="outline" size="sm" onclick={onClear}>
      Clear Filters
    </Button>
  {/if}
</div>

