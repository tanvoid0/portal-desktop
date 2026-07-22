<!--
	Frameworks & Languages Settings Page - Unified page with tabs for Languages, Package Managers, and Frameworks
-->

<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    Tabs,
    TabsList,
    TabsTrigger,
    TabsContent,
  } from "$lib/components/ui/tabs";
  import LanguagesSettings from "$lib/domains/settings/components/LanguagesSettings.svelte";
  import PackageManagersSettings from "$lib/domains/settings/components/PackageManagersSettings.svelte";
  import FrameworksSettings from "$lib/domains/settings/components/FrameworksSettings.svelte";

  // Get active tab from URL params or default to languages
  const activeTab = $derived.by(() => {
    const tab = $page.url.searchParams.get("tab");
    if (
      tab === "languages" ||
      tab === "package-managers" ||
      tab === "frameworks"
    ) {
      return tab;
    }
    return "languages"; // default
  });

  function handleTabChange(value: string) {
    goto(`/settings/frameworks-languages?tab=${value}`, {
      replaceState: true,
      noScroll: true,
    });
  }
</script>

<svelte:head>
  <title>Frameworks & Languages - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight">Frameworks & Languages</h2>
    <p class="text-muted-foreground">
      Manage your languages, frameworks, and package managers with intelligent
      recommendations
    </p>
  </div>

  <Tabs value={activeTab} onValueChange={handleTabChange} class="w-full">
    <TabsList class="grid w-full grid-cols-3">
      <TabsTrigger value="languages" class="flex items-center gap-2">
        <svg
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
          />
        </svg>
        Languages
      </TabsTrigger>
      <TabsTrigger value="package-managers" class="flex items-center gap-2">
        <svg
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
          />
        </svg>
        Package Managers
      </TabsTrigger>
      <TabsTrigger value="frameworks" class="flex items-center gap-2">
        <svg
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
          />
        </svg>
        Frameworks
      </TabsTrigger>
    </TabsList>

    <TabsContent value="languages" class="mt-6">
      <LanguagesSettings />
    </TabsContent>

    <TabsContent value="package-managers" class="mt-6">
      <PackageManagersSettings />
    </TabsContent>

    <TabsContent value="frameworks" class="mt-6">
      <FrameworksSettings />
    </TabsContent>
  </Tabs>
</div>
