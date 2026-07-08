<!--

  Shell sidebar layout — single scroll owner per column.

  Uses a nested SidebarProvider so domain sidebar state is independent

  from the main app navigation sidebar.

-->

<script lang="ts">

  import { onMount } from "svelte";

  import type { Snippet } from "svelte";

  import {

    Provider as SidebarProvider,

    Sidebar as SidebarRoot,

    SidebarContent,

    SidebarTrigger,

  } from "$lib/components/ui/sidebar";

  import { SIDEBAR_DOMAIN_COOKIE_NAME } from "$lib/components/ui/sidebar/constants.js";



  interface Props {

    sidebar: Snippet<[]>;

    children: Snippet<[]>;

    contentClass?: string;

    sidebarClass?: string;

    mainClass?: string;

    showMobileTrigger?: boolean;

    mobileTriggerLabel?: string;

  }



  let {

    sidebar,

    children,

    contentClass = "flex h-full min-h-0 w-full flex-1 overflow-hidden",

    sidebarClass = "flex h-full min-h-0 flex-col overflow-hidden",

    mainClass = "min-h-0 min-w-0 flex-1 overflow-hidden",

    showMobileTrigger = true,

    mobileTriggerLabel = "Section menu",

  }: Props = $props();



  let domainSidebarOpen = $state(true);



  function readDomainSidebarCookie(): boolean | null {

    if (typeof document === "undefined") return null;

    const match = document.cookie.match(

      new RegExp(`(?:^|; )${SIDEBAR_DOMAIN_COOKIE_NAME}=([^;]*)`),

    );

    if (!match) return null;

    if (match[1] === "true") return true;

    if (match[1] === "false") return false;

    return null;

  }



  onMount(() => {

    const cookieOpen = readDomainSidebarCookie();

    if (cookieOpen !== null) {

      domainSidebarOpen = cookieOpen;

    }

  });

</script>



<SidebarProvider

  bind:open={domainSidebarOpen}

  cookieName={SIDEBAR_DOMAIN_COOKIE_NAME}

  enableShortcut={false}

  style="--sidebar-width: var(--sidebar-width-domain, 14rem);"

  class="flex h-full min-h-0 w-full flex-col overflow-hidden"

>

  {#if showMobileTrigger}

    <div

      class="flex shrink-0 items-center gap-2 border-b bg-background px-3 py-2 md:hidden"

    >

      <SidebarTrigger class="size-8" />

      <span class="text-xs font-medium text-muted-foreground">{mobileTriggerLabel}</span>

    </div>

  {/if}



  <div class={contentClass}>

    <SidebarRoot collapsible="icon">

      <div class={sidebarClass}>

        <SidebarContent class="gap-0 p-0">

          {@render sidebar()}

        </SidebarContent>

      </div>

    </SidebarRoot>

    <main class={mainClass}>

      {@render children()}

    </main>

  </div>

</SidebarProvider>


