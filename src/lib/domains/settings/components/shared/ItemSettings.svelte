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
  import { rankRecommendedNames, mergeRecommendedGroup } from './recommendations';
  import ItemCardGrid from './ItemCardGrid.svelte';
  import ItemSuggestedList from './ItemSuggestedList.svelte';
  import ItemFormDialog from './ItemFormDialog.svelte';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { logger } from '$lib/domains/shared';
  import { suggestionEngine, learningService } from '$lib/domains/learning';
  import type {
    BaseItem,
    BaseSuggestedItem,
    BaseItemGroup,
    ItemService,
    SettingsComponentConfig,
  } from './types';

  interface Props<T extends BaseItem, TSuggested extends BaseSuggestedItem> {
    config: SettingsComponentConfig<T, TSuggested>;
  }

  let { config }: Props<T, TSuggested> = $props();

  const log = logger.createScoped(`${config.itemName}Settings`);

  // State
  let items = $state<T[]>([]);
  let suggestedGroups = $state<BaseItemGroup<TSuggested>[]>([]);
  let recommendedNames = $state<string[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let showAddDialog = $state(false);
  let editingItem = $state<T | null>(null);

  // Form state
  let itemName = $state('');
  let itemIcon = $state('');
  let itemIconType = $state<'devicon' | 'file'>('devicon');
  let itemCategory = $state('Custom');

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    isLoading = true;
    try {
      await Promise.all([loadItems(), loadSuggestedItems(), loadRecommendedItems()]);
    } catch (error) {
      log.error(`Failed to load ${config.itemName.toLowerCase()} data`, error);
      toast.error(`Failed to load ${config.itemNamePlural.toLowerCase()}`);
    } finally {
      isLoading = false;
    }
  }

  async function loadItems() {
    try {
      items = await config.service.getAll();
      log.info(`${config.itemNamePlural} loaded`, { count: items.length });
    } catch (error) {
      log.error(`Failed to load ${config.itemNamePlural.toLowerCase()}`, error);
      toast.error(`Failed to load ${config.itemNamePlural.toLowerCase()}`);
    }
  }

  async function loadSuggestedItems() {
    try {
      suggestedGroups = await config.service.getSuggested();
      log.info(`Suggested ${config.itemNamePlural.toLowerCase()} loaded`, {
        count: suggestedGroups.length,
      });
    } catch (error) {
      log.error(`Failed to load suggested ${config.itemNamePlural.toLowerCase()}`, error);
    }
  }

  async function loadRecommendedItems() {
    try {
      const suggestions = await suggestionEngine.getContextualSuggestions(
        config.recommendationPatternType
      );
      recommendedNames = rankRecommendedNames(suggestions, config.recommendationDataKey);
      log.info(`Recommended ${config.itemNamePlural.toLowerCase()} loaded`, {
        count: recommendedNames.length,
      });
    } catch (error) {
      log.error(`Failed to load recommended ${config.itemNamePlural.toLowerCase()}`, error);
    }
  }

  function startAddingItem(suggested?: TSuggested | string) {
    if (typeof suggested === 'string') {
      itemName = suggested || '';
      itemIcon = '';
      itemIconType = 'devicon';
      itemCategory = 'Custom';
    } else if (suggested) {
      itemName = suggested.name;
      itemIcon = suggested.icon;
      itemIconType = 'devicon';
      itemCategory = suggested.category;
    } else {
      itemName = '';
      itemIcon = '';
      itemIconType = 'devicon';
      itemCategory = 'Custom';
    }
    editingItem = null;
    showAddDialog = true;
  }

  function startEditingItem(item: T) {
    itemName = item.name;
    itemIcon = item.icon;
    itemIconType = item.icon_type;
    itemCategory = item.category;
    editingItem = item;
    showAddDialog = true;
  }

  async function saveItem() {
    if (!itemName.trim()) {
      toast.error(`${config.itemName} name is required`);
      return;
    }

    try {
      if (editingItem) {
        await config.service.update(
          editingItem.id,
          itemName.trim(),
          itemIcon.trim() || 'logos:visual-studio-code',
          itemIconType,
          itemCategory.trim() || 'Custom'
        );
        toast.success(`${config.itemName} updated successfully`);
      } else {
        const created = await config.service.create(
          itemName.trim(),
          itemIcon.trim() || 'logos:visual-studio-code',
          itemIconType,
          itemCategory.trim() || 'Custom'
        );
        toast.success(`${config.itemName} added successfully`);

        if (config.onItemAdded) {
          await config.onItemAdded(created);
        }
      }

      showAddDialog = false;
      await loadData();
    } catch (error) {
      log.error(`Failed to save ${config.itemName.toLowerCase()}`, error);
      toast.error(
        error instanceof Error ? error.message : `Failed to save ${config.itemName.toLowerCase()}`
      );
    }
  }

  async function deleteItem(id: number, name: string) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete the ${config.itemName.toLowerCase()} "${name}"?`,
      `Delete ${config.itemName.toLowerCase()}`
    );
    if (!confirmed) return;

    try {
      await config.service.delete(id);
      toast.success(`${config.itemName} deleted successfully`);
      await loadItems();
    } catch (error) {
      log.error(`Failed to delete ${config.itemName.toLowerCase()}`, error);
      toast.error(`Failed to delete ${config.itemName.toLowerCase()}`);
    }
  }

  function isItemInList(name: string): boolean {
    return items.some((item) => item.name.toLowerCase() === name.toLowerCase());
  }

  const filteredItems = $derived.by(() => {
    if (!searchQuery.trim()) return items;
    const query = searchQuery.toLowerCase();
    return items.filter(
      (item) =>
        item.name.toLowerCase().includes(query) ||
        item.category.toLowerCase().includes(query) ||
        item.icon.toLowerCase().includes(query)
    );
  });

  const allSuggestedItems = $derived.by(() => {
    const all: TSuggested[] = [];
    for (const group of suggestedGroups) {
      all.push(...group.items);
    }
    return all;
  });

  const mergedSuggestedGroups = $derived(
    mergeRecommendedGroup(suggestedGroups, allSuggestedItems, recommendedNames)
  );

  // Filter out already-added items from each group
  const filteredSuggestedGroups = $derived.by(() => {
    return mergedSuggestedGroups
      .map((group) => ({
        ...group,
        items: group.items.filter((item) => !isItemInList(item.name)),
      }))
      .filter((group) => group.items.length > 0);
  });

  // Calculate total count of available suggested items (after filtering)
  const totalSuggestedCount = $derived.by(() => {
    return filteredSuggestedGroups.reduce((sum, group) => sum + group.items.length, 0);
  });

  async function addAllItemsFromGroup(group: BaseItemGroup<TSuggested>) {
    const itemsToAdd = group.items.filter((item) => !isItemInList(item.name));

    if (itemsToAdd.length === 0) {
      toast.info(`All ${config.itemNamePlural.toLowerCase()} from this group are already added`);
      return;
    }

    try {
      isLoading = true;
      const result = await config.service.createBatch(itemsToAdd);

      if (result.success.length > 0) {
        toast.success(
          `Successfully added ${result.success.length} ${config.itemName.toLowerCase()}${result.success.length > 1 ? 's' : ''}`
        );

        if (config.onItemsBatchAdded) {
          await config.onItemsBatchAdded(result.success);
        }
      }

      if (result.failed.length > 0) {
        const failedNames = result.failed.map((f) => f.item.name).join(', ');
        toast.error(
          `Failed to add ${result.failed.length} ${config.itemName.toLowerCase()}${result.failed.length > 1 ? 's' : ''}: ${failedNames}`
        );
      }

      await loadData();
    } catch (error) {
      log.error(`Failed to add ${config.itemNamePlural.toLowerCase()} in batch`, error);
      toast.error(`Failed to add ${config.itemNamePlural.toLowerCase()}`);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <Input
        placeholder="Search {config.itemNamePlural.toLowerCase()}..."
        bind:value={searchQuery}
        class="w-64"
      >
        <Search class="h-4 w-4" />
      </Input>
    </div>
    <Button onclick={() => startAddingItem()}>
      <Plus class="mr-2 h-4 w-4" />
      Add {config.itemName}
    </Button>
  </div>

  {#if isLoading}
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
          Your {config.itemNamePlural} ({items.length})
        </TabsTrigger>
        <TabsTrigger value="suggested">
          Suggested ({totalSuggestedCount})
          {#if recommendedNames.length > 0}
            <Sparkles class="ml-1 h-3 w-3 text-primary" />
          {/if}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="your-items" class="space-y-4">
        <ItemCardGrid
          items={filteredItems}
          isSearching={!!searchQuery}
          itemName={config.itemName}
          emptyMessage={config.emptyMessage}
          emptySearchMessage={config.emptySearchMessage}
          emptyIcon={config.emptyIcon}
          onAdd={() => startAddingItem()}
          onEdit={startEditingItem}
          onDelete={deleteItem}
        />
      </TabsContent>

      <TabsContent value="suggested" class="space-y-4">
        <ItemSuggestedList
          groups={filteredSuggestedGroups}
          hasAnySuggestions={mergedSuggestedGroups.length > 0}
          itemName={config.itemName}
          itemNamePlural={config.itemNamePlural}
          emptyIcon={config.emptyIcon}
          {isLoading}
          onAddAll={addAllItemsFromGroup}
          onAdd={startAddingItem}
        />
      </TabsContent>
    </Tabs>
  {/if}
</div>

<ItemFormDialog
  bind:open={showAddDialog}
  itemLabel={config.itemName}
  editing={!!editingItem}
  bind:name={itemName}
  bind:category={itemCategory}
  bind:icon={itemIcon}
  bind:iconType={itemIconType}
  onSave={saveItem}
/>
