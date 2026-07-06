<!--
	Reusable Item Settings Component
	Supports Languages, Package Managers, and Frameworks
-->

<script
  lang="ts"
  generics="T extends BaseItem, TSuggested extends BaseSuggestedItem"
>
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import Select from "$lib/components/ui/select.svelte";
  import { Input } from "$lib/components/ui/input";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog";
  import {
    Tabs,
    TabsList,
    TabsTrigger,
    TabsContent,
  } from "$lib/components/ui/tabs";
  import {
    Plus,
    Edit,
    Trash2,
    Search,
    Loader2,
    Sparkles,
    TrendingUp,
  } from "@lucide/svelte";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import { logger } from "$lib/domains/shared";
  import { suggestionEngine, learningService } from "$lib/domains/learning";
  import Icon from "@iconify/svelte";
  import type {
    BaseItem,
    BaseSuggestedItem,
    BaseItemGroup,
    ItemService,
    SettingsComponentConfig,
  } from "./types";

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
  let searchQuery = $state("");
  let showAddDialog = $state(false);
  let editingItem = $state<T | null>(null);

  // Form state
  let itemName = $state("");
  let itemIcon = $state("");
  let itemIconType = $state<"devicon" | "file">("devicon");
  let itemCategory = $state("Custom");

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    isLoading = true;
    try {
      await Promise.all([
        loadItems(),
        loadSuggestedItems(),
        loadRecommendedItems(),
      ]);
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
      log.error(
        `Failed to load suggested ${config.itemNamePlural.toLowerCase()}`,
        error,
      );
    }
  }

  async function loadRecommendedItems() {
    try {
      const suggestions = await suggestionEngine.getContextualSuggestions(
        config.recommendationPatternType,
      );
      const scores = new Map<string, number>();
      for (const suggestion of suggestions) {
        if (
          suggestion.pattern_data &&
          typeof suggestion.pattern_data === "object"
        ) {
          const data = suggestion.pattern_data as Record<string, unknown>;
          const value = data[config.recommendationDataKey];
          if (value && typeof value === "string") {
            const score = suggestion.frequency * suggestion.success_rate;
            const currentScore = scores.get(value) || 0;
            scores.set(value, currentScore + score);
          }
        }
      }
      const sorted = Array.from(scores.entries())
        .sort((a, b) => b[1] - a[1])
        .map(([name]) => name)
        .slice(0, 15);
      recommendedNames = sorted;
      log.info(`Recommended ${config.itemNamePlural.toLowerCase()} loaded`, {
        count: recommendedNames.length,
      });
    } catch (error) {
      log.error(
        `Failed to load recommended ${config.itemNamePlural.toLowerCase()}`,
        error,
      );
    }
  }

  function startAddingItem(suggested?: TSuggested | string) {
    if (typeof suggested === "string") {
      itemName = suggested || "";
      itemIcon = "";
      itemIconType = "devicon";
      itemCategory = "Custom";
    } else if (suggested) {
      itemName = suggested.name;
      itemIcon = suggested.icon;
      itemIconType = "devicon";
      itemCategory = suggested.category;
    } else {
      itemName = "";
      itemIcon = "";
      itemIconType = "devicon";
      itemCategory = "Custom";
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
          itemIcon.trim() || "logos:code",
          itemIconType,
          itemCategory.trim() || "Custom",
        );
        toast.success(`${config.itemName} updated successfully`);
      } else {
        const created = await config.service.create(
          itemName.trim(),
          itemIcon.trim() || "logos:code",
          itemIconType,
          itemCategory.trim() || "Custom",
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
        error instanceof Error
          ? error.message
          : `Failed to save ${config.itemName.toLowerCase()}`,
      );
    }
  }

  async function deleteItem(id: number, name: string) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete the ${config.itemName.toLowerCase()} "${name}"?`,
      `Delete ${config.itemName.toLowerCase()}`,
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
        item.icon.toLowerCase().includes(query),
    );
  });

  const allSuggestedItems = $derived.by(() => {
    const all: TSuggested[] = [];
    for (const group of suggestedGroups) {
      all.push(...group.items);
    }
    return all;
  });

  // Merge recommended items into suggested groups
  const mergedSuggestedGroups = $derived.by(() => {
    const groups = [...suggestedGroups];

    if (recommendedNames.length > 0) {
      const recommendedItems: TSuggested[] = recommendedNames.map((name) => {
        const existing = allSuggestedItems.find(
          (si) => si.name.toLowerCase() === name.toLowerCase(),
        );

        if (existing) {
          return existing;
        }

        return {
          name,
          icon: `logos:${name.toLowerCase().replace(/\s+/g, "-")}`,
          category: "Recommended",
        } as TSuggested;
      });

      groups.unshift({
        category: "Recommended (Based on Your Usage)",
        items: recommendedItems,
      });
    }

    return groups;
  });

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
    return filteredSuggestedGroups.reduce(
      (sum, group) => sum + group.items.length,
      0,
    );
  });

  async function addAllItemsFromGroup(group: BaseItemGroup<TSuggested>) {
    const itemsToAdd = group.items.filter((item) => !isItemInList(item.name));

    if (itemsToAdd.length === 0) {
      toast.info(
        `All ${config.itemNamePlural.toLowerCase()} from this group are already added`,
      );
      return;
    }

    try {
      isLoading = true;
      const result = await config.service.createBatch(itemsToAdd);

      if (result.success.length > 0) {
        toast.success(
          `Successfully added ${result.success.length} ${config.itemName.toLowerCase()}${result.success.length > 1 ? "s" : ""}`,
        );

        if (config.onItemsBatchAdded) {
          await config.onItemsBatchAdded(result.success);
        }
      }

      if (result.failed.length > 0) {
        const failedNames = result.failed.map((f) => f.item.name).join(", ");
        toast.error(
          `Failed to add ${result.failed.length} ${config.itemName.toLowerCase()}${result.failed.length > 1 ? "s" : ""}: ${failedNames}`,
        );
      }

      await loadData();
    } catch (error) {
      log.error(
        `Failed to add ${config.itemNamePlural.toLowerCase()} in batch`,
        error,
      );
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
        {@const filtered = filteredItems}
        {#if filtered.length === 0}
          <Card>
            <CardContent
              class="flex flex-col items-center justify-center py-12"
            >
              <svelte:component
                this={config.emptyIcon}
                class="mb-4 h-12 w-12 text-muted-foreground"
              />
              <p class="mb-2 text-muted-foreground">
                {#if searchQuery}
                  {config.emptySearchMessage}
                {:else}
                  {config.emptyMessage}
                {/if}
              </p>
              <Button variant="outline" onclick={() => startAddingItem()}>
                <Plus class="mr-2 h-4 w-4" />
                Add Your First {config.itemName}
              </Button>
            </CardContent>
          </Card>
        {:else}
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {#each filtered as item}
              <Card class="relative">
                <CardHeader>
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-3">
                      {#if item.icon_type === "devicon"}
                        <Icon icon={item.icon} class="h-8 w-8" />
                      {:else}
                        <img src={item.icon} alt={item.name} class="h-8 w-8" />
                      {/if}
                      <div>
                        <CardTitle class="text-lg">{item.name}</CardTitle>
                        <CardDescription>{item.category}</CardDescription>
                      </div>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div class="mt-2 flex items-center justify-end gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => startEditingItem(item)}
                    >
                      <Edit class="mr-1 h-3 w-3" />
                      Edit
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => deleteItem(item.id, item.name)}
                      class="text-red-500 hover:text-red-700"
                    >
                      <Trash2 class="mr-1 h-3 w-3" />
                      Delete
                    </Button>
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      </TabsContent>

      <TabsContent value="suggested" class="space-y-4">
        {#if filteredSuggestedGroups.length === 0}
          <Card>
            <CardContent
              class="flex flex-col items-center justify-center py-12"
            >
              <svelte:component
                this={config.emptyIcon}
                class="mb-4 h-12 w-12 text-muted-foreground"
              />
              <p class="text-muted-foreground">
                {#if mergedSuggestedGroups.length === 0}
                  No suggested {config.itemNamePlural.toLowerCase()} available
                {:else}
                  All suggested {config.itemNamePlural.toLowerCase()} have been added
                {/if}
              </p>
            </CardContent>
          </Card>
        {:else}
          {#each filteredSuggestedGroups as group}
            <Card>
              <CardHeader>
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    {#if group.category.includes("Recommended")}
                      <Sparkles class="h-5 w-5 text-primary" />
                    {/if}
                    <div>
                      <CardTitle>{group.category}</CardTitle>
                      <CardDescription>
                        {group.items.length}
                        {config.itemName.toLowerCase()}{group.items.length > 1
                          ? "s"
                          : ""} available
                      </CardDescription>
                    </div>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={() => addAllItemsFromGroup(group)}
                    disabled={isLoading}
                  >
                    <Plus class="mr-2 h-4 w-4" />
                    Add All
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <div
                  class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3"
                >
                  {#each group.items as item}
                    <div
                      class="flex items-center justify-between rounded-lg border p-3 transition-colors hover:bg-accent/50"
                    >
                      <div class="flex min-w-0 flex-1 items-center gap-3">
                        <Icon icon={item.icon} class="h-6 w-6 flex-shrink-0" />
                        <div class="min-w-0 flex-1">
                          <div
                            class="flex items-center gap-1 truncate font-medium"
                          >
                            {item.name}
                            {#if group.category.includes("Recommended")}
                              <Badge
                                variant="outline"
                                class="border-primary bg-primary/5 text-xs text-primary"
                              >
                                <TrendingUp class="mr-1 h-3 w-3" />
                                Recommended
                              </Badge>
                            {/if}
                          </div>
                          <div class="text-xs text-muted-foreground">
                            {item.category}
                          </div>
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => startAddingItem(item)}
                      >
                        <Plus class="h-4 w-4" />
                      </Button>
                    </div>
                  {/each}
                </div>
              </CardContent>
            </Card>
          {/each}
        {/if}
      </TabsContent>
    </Tabs>
  {/if}
</div>

<Dialog bind:open={showAddDialog}>
  <DialogContent class="sm:max-w-[500px]">
    <DialogHeader>
      <DialogTitle>
        {editingItem ? `Edit ${config.itemName}` : `Add ${config.itemName}`}
      </DialogTitle>
      <DialogDescription>
        {editingItem
          ? `Update ${config.itemName.toLowerCase()} details`
          : `Add a new ${config.itemName.toLowerCase()} to your collection`}
      </DialogDescription>
    </DialogHeader>

    <div class="grid gap-4 py-4">
      <div class="space-y-2">
        <Label for="item-name">{config.itemName} Name *</Label>
        <Input
          id="item-name"
          bind:value={itemName}
          placeholder="Enter name..."
        />
      </div>
      <div class="space-y-2">
        <Label for="item-category">Category</Label>
        <Input
          id="item-category"
          bind:value={itemCategory}
          placeholder="Enter category..."
        />
      </div>
      <div class="space-y-2">
        <Label for="item-icon">Icon</Label>
        <Input
          id="item-icon"
          bind:value={itemIcon}
          placeholder="logos:example (for Devicon) or image URL"
        />
        <p class="text-xs text-muted-foreground">
          Enter a Devicon name (e.g., "logos:javascript") or an image URL
        </p>
      </div>
      <div class="space-y-2">
        <Label for="item-icon-type">Icon Type</Label>
        <Select
          bind:value={itemIconType}
          options={[
            { value: "devicon", label: "Devicon" },
            { value: "file", label: "Image URL" },
          ]}
        />
      </div>
    </div>

    <DialogFooter>
      <Button variant="outline" onclick={() => (showAddDialog = false)}
        >Cancel</Button
      >
      <Button onclick={saveItem}>
        {editingItem ? "Update" : "Add"}
        {config.itemName}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  select {
    background-color: hsl(var(--background));
    color: hsl(var(--foreground));
    border-color: hsl(var(--border));
  }
</style>
