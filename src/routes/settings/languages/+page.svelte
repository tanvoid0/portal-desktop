<!--
	Languages Settings Page
-->

<script lang="ts">
  import LanguagesSettings from "$lib/domains/settings/components/LanguagesSettings.svelte";
</script>

<svelte:head>
  <title>Languages - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight">Language Management</h2>
    <p class="text-muted-foreground">
      Manage your languages and view intelligent recommendations based on your
      usage
    </p>
  </div>

  <LanguagesSettings />
</div>
