<!--
	Theme Settings Page
-->

<script lang="ts">
  import { settings } from "$lib/domains/settings/stores/settingsStore";
  import ThemeCustomizer from "$lib/domains/settings/components/ThemeCustomizer.svelte";
  import ThemeDisplay from "$lib/components/ThemeDisplay.svelte";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { get } from "svelte/store";

  const settingsData = $derived($settings);

  function handleThemeUpdate(
    updates: Partial<import("$lib/domains/settings/types").ThemeSettings>,
  ) {
    const current = get(settings);
    if (!current) return;
    const newSettings = {
      ...current,
      theme: { ...current.theme, ...updates },
    };
    settings.set(newSettings);
  }
</script>

<svelte:head>
  <title>Theme Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight">Theme Settings</h2>
    <p class="text-muted-foreground">Customize appearance and color schemes</p>
  </div>

  {#if settingsData}
    <ThemeCustomizer
      settings={settingsData.theme}
      onUpdate={handleThemeUpdate}
    />

    <Card>
      <CardHeader>
        <CardTitle>Component Gallery</CardTitle>
        <CardDescription>
          Preview buttons, forms, alerts, and theme tokens with your current
          theme
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ThemeDisplay embedded />
      </CardContent>
    </Card>
  {/if}
</div>
