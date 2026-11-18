<!--
	Reusable Item Settings Component
	Supports Languages, Package Managers, and Frameworks
-->

<script lang="ts" generics="T extends BaseItem, TSuggested extends BaseSuggestedItem">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
  import Plus from '~icons/fa6-solid/plus';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import ItemCardGrid from './ItemCardGrid.svelte';
  import ItemSuggestedList from './ItemSuggestedList.svelte';
  import ItemFormDialog from './ItemFormDialog.svelte';
  import { ItemSettingsState } from './itemSettingsState.svelte';
  import type { BaseItem, BaseSuggestedItem, SettingsComponentConfig } from './types';

  interface Props<T extends BaseItem, TSuggested extends BaseSuggestedItem> {
    config: SettingsComponentConfig<T, TSuggested>;
  }

  let { config }: Props<T, TSuggested> = $props();

  const st = new ItemSettingsState<T, TSuggested>(() => config);

  onMount(st.loadData);
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <Input
        placeholder="Search {config.itemNamePlural.toLowerCase()}..."
        bind:value={st.searchQuery}
        class="w-64"
      >
        <Search class="h-4 w-4" />
      </Input>
    </div>
    <Button onclick={() => st.startAddingItem()}>
      <Plus class="mr-2 h-4 w-4" />
      Add {config.itemName}
    </Button>
  </div>

  {#if st.isLoading}
    <div class="flex items-center justify-center py-12">
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
      <span class="ml-2 text-muted-foreground"
        >Loading {config.itemNamePlural.toLowerCase()}...</span
      >
    </div>
  {:else}
    <Tabs value="your-items" class="space-y-4">
      <TabsList>
        <TabsTrigger value="your-items">
          Your {config.itemNamePlural} ({st.items.length})
        </TabsTrigger>
        <TabsTrigger value="suggested">
          Suggested ({st.totalSuggestedCount})
          {#if st.recommendedNames.length > 0}
            <Sparkles class="ml-1 h-3 w-3 text-primary" />
          {/if}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="your-items" class="space-y-4">
        <ItemCardGrid
          items={st.filteredItems}
          isSearching={!!st.searchQuery}
          itemName={config.itemName}
          emptyMessage={config.emptyMessage}
          emptySearchMessage={config.emptySearchMessage}
          emptyIcon={config.emptyIcon}
          onAdd={() => st.startAddingItem()}
          onEdit={st.startEditingItem}
          onDelete={st.deleteItem}
        />
      </TabsContent>

      <TabsContent value="suggested" class="space-y-4">
        <ItemSuggestedList
          groups={st.filteredSuggestedGroups}
          hasAnySuggestions={st.mergedSuggestedGroups.length > 0}
          itemName={config.itemName}
          itemNamePlural={config.itemNamePlural}
          emptyIcon={config.emptyIcon}
          isLoading={st.isLoading}
          onAddAll={st.addAllItemsFromGroup}
          onAdd={st.startAddingItem}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>

<ItemFormDialog
  bind:open={st.showAddDialog}
  itemLabel={config.itemName}
  editing={!!st.editingItem}
  bind:name={st.itemName}
  bind:category={st.itemCategory}
  bind:icon={st.itemIcon}
  bind:iconType={st.itemIconType}
  onSave={st.saveItem}
/>
