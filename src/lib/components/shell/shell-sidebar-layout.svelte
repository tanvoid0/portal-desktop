<script lang="ts">
  import type { Snippet } from "svelte";
  import { Sidebar as SidebarRoot } from "$lib/components/ui/sidebar";

  interface Props {
    sidebar: Snippet<[]>;
    children: Snippet<[]>;
    contentClass?: string;
    sidebarClass?: string;
    mainClass?: string;
  }

  let {
    sidebar,
    children,
    contentClass = "flex h-full min-h-0 w-full flex-1 overflow-hidden",
    sidebarClass = "flex h-full min-h-0 flex-col overflow-hidden",
    mainClass = "min-h-0 min-w-0 flex-1 overflow-y-auto",
  }: Props = $props();
</script>

<div class="flex h-full min-h-0 w-full flex-col overflow-hidden">
  <div class={contentClass}>
    <SidebarRoot collapsible="icon">
      <div class={sidebarClass}>
        {@render sidebar()}
      </div>
    </SidebarRoot>
    <main class={mainClass}>
      {@render children()}
    </main>
  </div>
</div>
