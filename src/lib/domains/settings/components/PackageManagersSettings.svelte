<!--
	Package Managers Settings - Manage package managers with intelligent recommendations
-->

<script lang="ts">
  import { Package } from "@lucide/svelte";
  import { learningService } from "$lib/domains/learning";
  import ItemSettings from "./shared/ItemSettings.svelte";
  import { packageManagerServiceAdapter } from "./shared/serviceAdapters";
  import type {
    PackageManager,
    SuggestedPackageManager,
  } from "$lib/domains/package_managers/types";

  const config = {
    itemName: "Package Manager",
    itemNamePlural: "Package Managers",
    emptyIcon: Package,
    emptyMessage: "No package managers added yet",
    emptySearchMessage: "No package managers match your search",
    service: packageManagerServiceAdapter,
    recommendationPatternType: "config" as const,
    recommendationDataKey: "package_manager",
    onItemAdded: async (item: PackageManager) => {
      try {
        await learningService.learnPattern({
          pattern_type: "config",
          pattern_data: {
            package_manager: item.name,
            category: item.category,
          },
          context: `category_${item.category.toLowerCase().replace(/\s+/g, "_")}`,
        });
      } catch (error) {
        console.warn("Failed to learn package manager pattern", error);
      }
    },
    onItemsBatchAdded: async (items: PackageManager[]) => {
      for (const item of items) {
        try {
          await learningService.learnPattern({
            pattern_type: "config",
            pattern_data: {
              package_manager: item.name,
              category: item.category,
            },
            context: `category_${item.category.toLowerCase().replace(/\s+/g, "_")}`,
          });
        } catch (error) {
          console.warn("Failed to learn package manager pattern", error);
        }
      }
    },
  };
</script>

<ItemSettings {config} />
