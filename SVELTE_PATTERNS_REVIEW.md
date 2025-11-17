# Svelte 5 Patterns Review - Portal Desktop

## Overview
This document reviews how the project uses Svelte 5 runes for state management, navigation, and lifecycle hooks.

## State Management Patterns

### ✅ Current Patterns (Svelte 5 Runes)

#### 1. **Component Props** - Using `$props()`
```typescript
// ✅ Correct pattern
interface Props {
  pods: ICloudResource[];
  onViewPod?: (pod: ICloudResource) => void;
  emptyMessage?: string;
}

let {
  pods = [],
  onViewPod,
  emptyMessage = 'No pods found'
}: Props = $props();
```

**Used in:**
- `PodsTable.svelte`
- `BaseResourceTable.svelte`
- `PodsFilters.svelte`
- `PodsStatistics.svelte`
- Most components in the project

#### 2. **Local State** - Using `$state()`
```typescript
// ✅ Correct pattern
let searchQuery = $state('');
let statusFilter = $state('');
let isLoading = $state(false);
let error = $state<string | null>(null);
```

**Used in:**
- `+page.svelte` files for local component state
- Form inputs and filters
- Loading states and error handling

#### 3. **Derived State** - Using `$derived()`
```typescript
// ✅ Correct pattern - Simple derived
const filteredPods = $derived(
  $cloudStore.resources[ResourceType.POD].filter(pod => {
    const matchesSearch = !searchQuery || pod.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = !statusFilter || pod.status === statusFilter;
    return matchesSearch && matchesStatus;
  })
);

// ✅ Correct pattern - Complex derived with .by()
const defaultColumns = $derived.by(() => {
  if (columns.length > 0) return columns;
  return [
    { key: 'name', label: 'Name', width: 'w-1/3' },
    // ...
  ];
});

// ✅ Correct pattern - Object derived
const stats = $derived({
  total: pods.length,
  running: pods.filter(p => p.status === ResourceStatus.RUNNING).length,
  pending: pods.filter(p => p.status === ResourceStatus.PENDING).length
});
```

**Used in:**
- Computed values from stores
- Filtered/transformed data
- Statistics and aggregations

#### 4. **Effects** - Using `$effect()`
```typescript
// ✅ Correct pattern - Side effects
$effect(() => {
  if (initialShell && initialShell !== currentShell && isInitialized) {
    switchShell(initialShell);
  }
});

// ✅ Correct pattern - Cleanup
$effect(() => {
  const timer = setInterval(() => {
    // do something
  }, 1000);
  
  return () => {
    clearInterval(timer);
  };
});
```

**Used in:**
- Reactive side effects
- Syncing external state
- Cleanup operations

## Navigation Patterns

### ✅ Using `goto()` from `$app/navigation`
```typescript
import { goto } from '$app/navigation';

// ✅ Correct pattern
function handlePodClick(pod: ICloudResource) {
  if (onViewPod) {
    onViewPod(pod);
  } else {
    goto(`/cloud/workloads/pods/${pod.name}?namespace=${pod.namespace}`);
  }
}

// ✅ With button click handlers
<Button onclick={() => goto('/cloud')}>Go to Cloud</Button>
```

**Used in:**
- Navigation from button clicks
- Programmatic navigation after actions
- Redirects and route changes

### ✅ Using `$page` store for route data
```typescript
import { page } from '$app/stores';

// ✅ Correct pattern
let projectId = $derived(parseInt($page.params.id || '0'));
let isSdkPage = $derived($page.url.pathname.startsWith('/sdk'));

onMount(async () => {
  if (isNaN(projectId)) {
    error = "Invalid project ID";
    return;
  }
  await loadProjectData();
});
```

**Used in:**
- Getting route parameters
- Checking current route
- Conditional logic based on route

## Lifecycle Hooks Patterns

### ✅ Using `onMount()`
```typescript
import { onMount } from 'svelte';

// ✅ Correct pattern - Async operations
onMount(async () => {
  if ($cloudStore.connection.isConnected) {
    await loadResources(ResourceType.POD);
  }
});

// ✅ Correct pattern - With cleanup
onMount(() => {
  const unsubscribe = store.subscribe(value => {
    // handle changes
  });
  
  return () => {
    unsubscribe();
  };
});
```

**Used in:**
- Loading initial data
- Setting up subscriptions
- Initializing components
- API calls on component mount

### ✅ Using `onDestroy()`
```typescript
import { onDestroy } from 'svelte';

// ✅ Correct pattern
onDestroy(() => {
  // Cleanup resources
  if (timer) clearInterval(timer);
  if (subscription) subscription.unsubscribe();
});
```

**Used in:**
- Cleaning up timers
- Unsubscribing from stores
- Removing event listeners
- Resource cleanup

## Store Integration Patterns

### ✅ Using Svelte Stores with Runes
```typescript
import { cloudStore } from '$lib/domains/cloud/stores';

// ✅ Correct pattern - Accessing store values
{#if !$cloudStore.connection.isConnected}
  <Card>Not Connected</Card>
{/if}

// ✅ Correct pattern - Derived from stores
const filteredPods = $derived(
  $cloudStore.resources[ResourceType.POD].filter(...)
);

// ✅ Correct pattern - In onMount
onMount(async () => {
  if ($cloudStore.connection.isConnected) {
    await loadResources(ResourceType.POD);
  }
});
```

**Used in:**
- Global state management
- Reactive UI updates
- Cross-component communication

## Common Patterns Summary

### ✅ Component Structure
```typescript
<script lang="ts">
  // 1. Imports
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  
  // 2. Props
  interface Props {
    // ...
  }
  let { ... }: Props = $props();
  
  // 3. Local state
  let isLoading = $state(false);
  
  // 4. Derived state
  const computed = $derived(...);
  
  // 5. Effects (if needed)
  $effect(() => { ... });
  
  // 6. Lifecycle
  onMount(async () => { ... });
  
  // 7. Functions
  function handleClick() { ... }
</script>
```

### ✅ Event Handlers
```typescript
// ✅ Direct function reference
<Button onclick={handleRefresh}>Refresh</Button>

// ✅ Inline arrow function
<Button onclick={() => goto('/cloud')}>Go to Cloud</Button>

// ✅ With parameters
<Button onclick={(e) => { e.stopPropagation(); onDelete(resource); }}>
  Delete
</Button>
```

### ✅ Conditional Rendering
```typescript
// ✅ Simple if/else
{#if !$cloudStore.connection.isConnected}
  <Card>Not Connected</Card>
{:else}
  <PodsTable pods={filteredPods} />
{/if}

// ✅ With derived values
{#if hasFilters}
  <Button onclick={onClear}>Clear Filters</Button>
{/if}
```

## TypeScript Configuration

### Current Setup
- ✅ Using `moduleResolution: "bundler"` (SvelteKit default)
- ✅ `types: ["svelte"]` in tsconfig.json
- ✅ Extending `.svelte-kit/tsconfig.json`

### Potential Issues
- ⚠️ TypeScript may need explicit Svelte module declarations
- ⚠️ Ensure `skipLibCheck: true` is set (already set)

## Best Practices Observed

1. ✅ **Consistent use of runes** - All components use `$state()`, `$derived()`, `$props()`
2. ✅ **Proper lifecycle management** - Using `onMount()` for initialization
3. ✅ **Clean navigation** - Using `goto()` from `$app/navigation`
4. ✅ **Store integration** - Properly accessing stores with `$` prefix
5. ✅ **Type safety** - Using TypeScript interfaces for props

## Migration Notes

The project has successfully migrated to Svelte 5 runes:
- ✅ No `export let` statements (all use `$props()`)
- ✅ No `$:` reactive statements (all use `$derived()`)
- ✅ No `let` reactive variables (all use `$state()`)
- ✅ Proper use of `$effect()` for side effects

