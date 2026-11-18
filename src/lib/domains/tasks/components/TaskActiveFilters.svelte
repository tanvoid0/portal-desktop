<script lang="ts">
  import FaMagnifyingGlass from '~icons/fa6-solid/magnifying-glass';
  import FaCircle from '~icons/fa6-solid/circle';
  import FaFlag from '~icons/fa6-solid/flag';
  import FaFilterCircleXmark from '~icons/fa6-solid/filter-circle-xmark';
  import { Button } from '$lib/components/ui/button';
  import { taskStatusLabel, type TaskStatus, type TaskPriority } from '../types';
  import TaskFilterChip from './TaskFilterChip.svelte';

  interface Props {
    searchQuery: string;
    statusFilters: TaskStatus[];
    priorityFilters: TaskPriority[];
    typeFilters: string[];
    onClearSearch: () => void;
    onToggleStatus: (s: TaskStatus) => void;
    onTogglePriority: (p: TaskPriority) => void;
    onToggleType: (t: string) => void;
    onClearAll: () => void;
  }
  let {
    searchQuery,
    statusFilters,
    priorityFilters,
    typeFilters,
    onClearSearch,
    onToggleStatus,
    onTogglePriority,
    onToggleType,
    onClearAll,
  }: Props = $props();
</script>

{#if searchQuery.trim() !== ''}
  <TaskFilterChip removeLabel="Clear search" onRemove={onClearSearch}>
    <FaMagnifyingGlass class="mr-1 h-3 w-3" />
    "{searchQuery}"
  </TaskFilterChip>
{/if}

{#each statusFilters as status (status)}
  <TaskFilterChip removeLabel="Remove status filter" onRemove={() => onToggleStatus(status)}>
    <FaCircle class="mr-1 h-3 w-3" />
    {taskStatusLabel(status)}
  </TaskFilterChip>
{/each}

{#each priorityFilters as priority (priority)}
  <TaskFilterChip removeLabel="Remove priority filter" onRemove={() => onTogglePriority(priority)}>
    <FaFlag
      class="mr-1 h-3 w-3 {priority === 'high'
        ? 'text-red-500'
        : priority === 'medium'
          ? 'text-yellow-500'
          : 'text-green-500'}"
    />
    {priority.charAt(0).toUpperCase() + priority.slice(1)}
  </TaskFilterChip>
{/each}

{#each typeFilters as type (type)}
  <TaskFilterChip removeLabel="Remove type filter" onRemove={() => onToggleType(type)}>
    {type}
  </TaskFilterChip>
{/each}

<Button variant="ghost" size="sm" onclick={onClearAll} class="h-7 text-xs">
  <FaFilterCircleXmark class="mr-1 h-3 w-3" />
  Clear All
</Button>
