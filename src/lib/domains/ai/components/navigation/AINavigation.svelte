<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import NavSectionList from "$lib/components/shell/nav-section-list.svelte";
  import { buildAiNavSections } from "$lib/config/ai-nav";

  const sections = buildAiNavSections();
</script>

<NavSectionList
  sections={sections}
  currentPath={$page.url.pathname}
  onNavigate={(url) => goto(url)}
/>
