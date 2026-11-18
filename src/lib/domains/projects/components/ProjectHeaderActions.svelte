<script lang="ts">
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Star from '~icons/fa6-solid/star';
  import Terminal from '~icons/fa6-solid/terminal';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import Info from '~icons/fa6-solid/circle-info';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Rocket from '~icons/fa6-solid/rocket';
  import Code from '~icons/fa6-solid/code';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';

  interface Props {
    starred: boolean;
    githubUrl: string | null;
    isDeploying: boolean;
    refreshing: boolean;
    /** Rail is docked beside the page (Overview, md+): the Details button is only for the Sheet. */
    railDocked?: boolean;
    onShowDetails: () => void;
    onOpenEditor: () => void;
    onOpenFolder: () => void;
    onTerminal: () => void;
    onEdit: () => void;
    onDeploy: () => void;
    onToggleStar: () => void;
    onRedetect: () => void;
  }
  let {
    starred,
    githubUrl,
    isDeploying,
    refreshing,
    railDocked = false,
    onShowDetails,
    onOpenEditor,
    onOpenFolder,
    onTerminal,
    onEdit,
    onDeploy,
    onToggleStar,
    onRedetect,
  }: Props = $props();
</script>

<Button
  variant="outline"
  onclick={onShowDetails}
  class="gap-2 {railDocked ? 'md:hidden' : ''}"
  aria-label="Project details"
>
  <Info class="h-4 w-4" />
  Details
</Button>
<Button onclick={onOpenEditor} class="gap-2">
  <Code class="h-4 w-4" />
  Open in editor
</Button>
<DropdownMenu>
  <DropdownMenuTrigger class={buttonVariants({ variant: 'outline' })} aria-label="More actions">
    <Ellipsis class="h-4 w-4" />
    More
  </DropdownMenuTrigger>
  <DropdownMenuContent align="end">
    <DropdownMenuItem onSelect={onOpenFolder}>
      <FolderOpen class="h-4 w-4" />
      Open folder
    </DropdownMenuItem>
    <DropdownMenuItem onSelect={onTerminal}>
      <Terminal class="h-4 w-4" />
      Terminal
    </DropdownMenuItem>
    <DropdownMenuItem onSelect={onEdit}>
      <Edit class="h-4 w-4" />
      Edit
    </DropdownMenuItem>
    <DropdownMenuItem onSelect={onDeploy} disabled={isDeploying}>
      {#if isDeploying}
        <Loader2 class="h-4 w-4 animate-spin" />
        Deploying…
      {:else}
        <Rocket class="h-4 w-4" />
        Deploy
      {/if}
    </DropdownMenuItem>
    <DropdownMenuItem onSelect={onToggleStar}>
      <Star class="h-4 w-4 {starred ? 'fill-current' : ''}" />
      {starred ? 'Unstar' : 'Star'}
    </DropdownMenuItem>
    <DropdownMenuItem onSelect={onRedetect} disabled={refreshing}>
      <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
      Re-detect stack
    </DropdownMenuItem>
    {#if githubUrl}
      <DropdownMenuItem>
        {#snippet child({ props })}
          <a {...props} href={githubUrl} target="_blank" rel="noreferrer">
            <ExternalLink class="h-4 w-4" />
            GitHub
          </a>
        {/snippet}
      </DropdownMenuItem>
    {/if}
  </DropdownMenuContent>
</DropdownMenu>
