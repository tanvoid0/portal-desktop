<!-- Layout frame shared by every coder provider (Portal and the agent backends): session list,
     header, banner, body + composer, and the workspace panel/sidebar. Layout only — each provider
     fills the snippets. Owns the session-list docking and the commit dialog. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import PanelLeftOpen from '~icons/fa6-solid/table-columns';
  import { useCoderWorkspace } from '../state/coderWorkspaceStore.svelte.js';
  import GitCommitDialog from './GitCommitDialog.svelte';

  interface Props {
    sessionsOpen?: boolean;
    commitOpen?: boolean;
    workspaceRoot: string;
    onCommitted?: () => void;
    /** Fold the session list into a Sheet when the pane (not the window) is too narrow to dock it. */
    adaptiveDock?: boolean;
    sessions: Snippet;
    header?: Snippet;
    banner?: Snippet;
    body: Snippet;
    composer?: Snippet;
    /** Omit when no panel is showing; the sidebar shows when the workspace store says so. */
    workspacePanel?: Snippet;
    workspaceSidebar?: Snippet;
  }

  let {
    sessionsOpen = $bindable(false),
    commitOpen = $bindable(false),
    workspaceRoot,
    onCommitted,
    adaptiveDock = false,
    sessions,
    header,
    banner,
    body,
    composer,
    workspacePanel,
    workspaceSidebar,
  }: Props = $props();

  const store = useCoderWorkspace();

  // The columns share the pane, not the window (the app nav takes ~16rem of it). The session list
  // (w-80) docks only while chat keeps 24rem beside the sidebar (w-56) and, when open, an equal
  // panel: 20 + 24 + 14 = 58rem, 82rem with the panel. Otherwise it folds into a Sheet. That is a
  // mounted component, not a style, so the pane is measured instead of using a container query.
  // ponytail: assumes the sidebar is open and never folds the panel/sidebar, so below ~48rem (the
  // 1024px min window leaves exactly that) the chat's min-w floor squeezes the panel; Sheet them too.
  let paneWidth = $state(0);
  const dockList = $derived(
    !adaptiveDock || paneWidth >= (workspacePanel && store.activePanel !== 'chat' ? 82 : 58) * 16
  );
</script>

<div class="flex h-full w-full overflow-hidden" bind:clientWidth={paneWidth}>
  <ResponsivePanel
    bind:open={sessionsOpen}
    side="left"
    collapsed={!dockList}
    desktopClass="w-80 bg-muted/20"
  >
    {@render sessions()}
  </ResponsivePanel>

  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    {#if !dockList}
      <div class="divider-edge-b flex shrink-0 items-center gap-2 px-2 py-1.5">
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8"
          title="Sessions"
          onclick={() => (sessionsOpen = true)}
        >
          <PanelLeftOpen class="h-4 w-4" />
        </Button>
      </div>
    {/if}
    {@render header?.()}
    {@render banner?.()}

    <div class="flex min-h-0 flex-1 overflow-hidden">
      <!-- min-w-80: the floor chat never drops below; the panel (min-w-0) gives way first -->
      <div class={['flex flex-1 flex-col overflow-hidden', adaptiveDock ? 'min-w-80' : 'min-w-0']}>
        {@render body()}
        {@render composer?.()}
      </div>

      {@render workspacePanel?.()}

      {#if workspaceSidebar && store.sidebarOpen}
        <ResponsivePanel
          bind:open={store.sidebarOpen}
          side="right"
          desktopClass="divider-edge-l w-56 bg-muted/20"
          borderClass=""
        >
          {@render workspaceSidebar()}
        </ResponsivePanel>
      {/if}
    </div>
  </div>
</div>

<GitCommitDialog bind:open={commitOpen} {workspaceRoot} {onCommitted} />
