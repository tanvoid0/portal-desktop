<!--
	Frameworks Settings - Manage frameworks with intelligent recommendations
-->

<script lang="ts">
  import { Package } from "@lucide/svelte";
  import { learningService } from "$lib/domains/learning";
  import ItemSettings from "./shared/ItemSettings.svelte";
  import { frameworkServiceAdapter } from "./shared/serviceAdapters";
  import type {
    Framework,
    SuggestedFramework,
  } from "$lib/domains/ide/services/ideService";

  const config = {
    itemName: "Framework",
    itemNamePlural: "Frameworks",
    emptyIcon: Package,
    emptyMessage: "No frameworks added yet",
    emptySearchMessage: "No frameworks match your search",
    service: frameworkServiceAdapter,
    recommendationPatternType: "framework" as const,
    recommendationDataKey: "framework",
    onItemAdded: async (item: Framework) => {
      try {
        await learningService.learnPattern({
          pattern_type: "framework",
          pattern_data: {
            framework: item.name,
            category: item.category,
          },
          context: `category_${item.category.toLowerCase().replace(/\s+/g, "_")}`,
        });
      } catch (error) {
        console.warn("Failed to learn framework pattern", error);
      }
    },
    onItemsBatchAdded: async (items: Framework[]) => {
      for (const item of items) {
        try {
          await learningService.learnPattern({
            pattern_type: "framework",
            pattern_data: {
              framework: item.name,
              category: item.category,
            },
            context: `category_${item.category.toLowerCase().replace(/\s+/g, "_")}`,
          });
        } catch (error) {
          console.warn("Failed to learn framework pattern", error);
        }
      }
    },
  };
</script>

<ItemSettings {config} />
