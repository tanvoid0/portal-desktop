import { rankRecommendedNames, mergeRecommendedGroup } from './recommendations';
import { toast } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import { logger } from '$lib/domains/shared';
import { suggestionEngine } from '$lib/domains/learning';
import type { BaseItem, BaseSuggestedItem, BaseItemGroup, SettingsComponentConfig } from './types';

/** Load/search/add/edit/delete state behind ItemSettings. */
export class ItemSettingsState<T extends BaseItem, TSuggested extends BaseSuggestedItem> {
  items = $state<T[]>([]);
  suggestedGroups = $state<BaseItemGroup<TSuggested>[]>([]);
  recommendedNames = $state<string[]>([]);
  isLoading = $state(false);
  searchQuery = $state('');
  showAddDialog = $state(false);
  editingItem = $state<T | null>(null);

  // Form state
  itemName = $state('');
  itemIcon = $state('');
  itemIconType = $state<'devicon' | 'file'>('devicon');
  itemCategory = $state('Custom');

  private log: ReturnType<typeof logger.createScoped>;

  constructor(private getConfig: () => SettingsComponentConfig<T, TSuggested>) {
    this.log = logger.createScoped(`${getConfig().itemName}Settings`);
  }

  private get config() {
    return this.getConfig();
  }

  filteredItems = $derived.by(() => {
    if (!this.searchQuery.trim()) return this.items;
    const query = this.searchQuery.toLowerCase();
    return this.items.filter(
      (item) =>
        item.name.toLowerCase().includes(query) ||
        item.category.toLowerCase().includes(query) ||
        item.icon.toLowerCase().includes(query)
    );
  });

  private allSuggestedItems = $derived.by(() => {
    const all: TSuggested[] = [];
    for (const group of this.suggestedGroups) {
      all.push(...group.items);
    }
    return all;
  });

  mergedSuggestedGroups = $derived(
    mergeRecommendedGroup(this.suggestedGroups, this.allSuggestedItems, this.recommendedNames)
  );

  // Filter out already-added items from each group
  filteredSuggestedGroups = $derived.by(() => {
    return this.mergedSuggestedGroups
      .map((group) => ({
        ...group,
        items: group.items.filter((item) => !this.isItemInList(item.name)),
      }))
      .filter((group) => group.items.length > 0);
  });

  // Calculate total count of available suggested items (after filtering)
  totalSuggestedCount = $derived.by(() => {
    return this.filteredSuggestedGroups.reduce((sum, group) => sum + group.items.length, 0);
  });

  loadData = async () => {
    this.isLoading = true;
    try {
      await Promise.all([this.loadItems(), this.loadSuggestedItems(), this.loadRecommendedItems()]);
    } catch (error) {
      this.log.error(`Failed to load ${this.config.itemName.toLowerCase()} data`, error);
      toast.error(`Failed to load ${this.config.itemNamePlural.toLowerCase()}`);
    } finally {
      this.isLoading = false;
    }
  };

  private async loadItems() {
    const { config } = this;
    try {
      this.items = await config.service.getAll();
      this.log.info(`${config.itemNamePlural} loaded`, { count: this.items.length });
    } catch (error) {
      this.log.error(`Failed to load ${config.itemNamePlural.toLowerCase()}`, error);
      toast.error(`Failed to load ${config.itemNamePlural.toLowerCase()}`);
    }
  }

  private async loadSuggestedItems() {
    const { config } = this;
    try {
      this.suggestedGroups = await config.service.getSuggested();
      this.log.info(`Suggested ${config.itemNamePlural.toLowerCase()} loaded`, {
        count: this.suggestedGroups.length,
      });
    } catch (error) {
      this.log.error(`Failed to load suggested ${config.itemNamePlural.toLowerCase()}`, error);
    }
  }

  private async loadRecommendedItems() {
    const { config } = this;
    try {
      const suggestions = await suggestionEngine.getContextualSuggestions(
        config.recommendationPatternType
      );
      this.recommendedNames = rankRecommendedNames(suggestions, config.recommendationDataKey);
      this.log.info(`Recommended ${config.itemNamePlural.toLowerCase()} loaded`, {
        count: this.recommendedNames.length,
      });
    } catch (error) {
      this.log.error(`Failed to load recommended ${config.itemNamePlural.toLowerCase()}`, error);
    }
  }

  startAddingItem = (suggested?: TSuggested | string) => {
    if (typeof suggested === 'string') {
      this.itemName = suggested || '';
      this.itemIcon = '';
      this.itemIconType = 'devicon';
      this.itemCategory = 'Custom';
    } else if (suggested) {
      this.itemName = suggested.name;
      this.itemIcon = suggested.icon;
      this.itemIconType = 'devicon';
      this.itemCategory = suggested.category;
    } else {
      this.itemName = '';
      this.itemIcon = '';
      this.itemIconType = 'devicon';
      this.itemCategory = 'Custom';
    }
    this.editingItem = null;
    this.showAddDialog = true;
  };

  startEditingItem = (item: T) => {
    this.itemName = item.name;
    this.itemIcon = item.icon;
    this.itemIconType = item.icon_type;
    this.itemCategory = item.category;
    this.editingItem = item;
    this.showAddDialog = true;
  };

  saveItem = async () => {
    const { config } = this;
    if (!this.itemName.trim()) {
      toast.error(`${config.itemName} name is required`);
      return;
    }

    try {
      if (this.editingItem) {
        await config.service.update(
          this.editingItem.id,
          this.itemName.trim(),
          this.itemIcon.trim() || 'logos:visual-studio-code',
          this.itemIconType,
          this.itemCategory.trim() || 'Custom'
        );
        toast.success(`${config.itemName} updated successfully`);
      } else {
        const created = await config.service.create(
          this.itemName.trim(),
          this.itemIcon.trim() || 'logos:visual-studio-code',
          this.itemIconType,
          this.itemCategory.trim() || 'Custom'
        );
        toast.success(`${config.itemName} added successfully`);

        if (config.onItemAdded) {
          await config.onItemAdded(created);
        }
      }

      this.showAddDialog = false;
      await this.loadData();
    } catch (error) {
      this.log.error(`Failed to save ${config.itemName.toLowerCase()}`, error);
      toast.error(
        error instanceof Error ? error.message : `Failed to save ${config.itemName.toLowerCase()}`
      );
    }
  };

  deleteItem = async (id: number, name: string) => {
    const { config } = this;
    const confirmed = await confirmAction(
      `Are you sure you want to delete the ${config.itemName.toLowerCase()} "${name}"?`,
      `Delete ${config.itemName.toLowerCase()}`
    );
    if (!confirmed) return;

    try {
      await config.service.delete(id);
      toast.success(`${config.itemName} deleted successfully`);
      await this.loadItems();
    } catch (error) {
      this.log.error(`Failed to delete ${config.itemName.toLowerCase()}`, error);
      toast.error(`Failed to delete ${config.itemName.toLowerCase()}`);
    }
  };

  private isItemInList(name: string): boolean {
    return this.items.some((item) => item.name.toLowerCase() === name.toLowerCase());
  }

  addAllItemsFromGroup = async (group: BaseItemGroup<TSuggested>) => {
    const { config } = this;
    const itemsToAdd = group.items.filter((item) => !this.isItemInList(item.name));

    if (itemsToAdd.length === 0) {
      toast.info(`All ${config.itemNamePlural.toLowerCase()} from this group are already added`);
      return;
    }

    try {
      this.isLoading = true;
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

      await this.loadData();
    } catch (error) {
      this.log.error(`Failed to add ${config.itemNamePlural.toLowerCase()} in batch`, error);
      toast.error(`Failed to add ${config.itemNamePlural.toLowerCase()}`);
    } finally {
      this.isLoading = false;
    }
  };
}
