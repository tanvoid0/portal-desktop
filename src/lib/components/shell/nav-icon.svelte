<script lang="ts">
  import Bot from '~icons/fa6-solid/robot';
  import Boxes from '~icons/fa6-solid/boxes-stacked';
  import CheckSquare from '~icons/fa6-solid/square-check';
  import Cloud from '~icons/fa6-solid/cloud';
  import Code from '~icons/fa6-solid/code';
  import Database from '~icons/fa6-solid/database';
  import FileCode from '~icons/fa6-solid/file-code';
  import FileText from '~icons/fa6-solid/file-lines';
  import Folder from '~icons/fa6-solid/folder';
  import FolderGit2 from '~icons/fa6-solid/folder-tree';
  import Globe from '~icons/fa6-solid/globe';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import History from '~icons/fa6-solid/clock-rotate-left';
  import Home from '~icons/fa6-solid/house';
  import Lock from '~icons/fa6-solid/lock';
  import MessageSquare from '~icons/fa6-solid/message';
  import Rocket from '~icons/fa6-solid/rocket';
  import ScanSearch from '~icons/fa6-solid/magnifying-glass';
  import Settings from '~icons/fa6-solid/gear';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import Terminal from '~icons/fa6-solid/terminal';
  import Variable from '~icons/fa6-solid/square-root-variable';
  import Workflow from '~icons/fa6-solid/diagram-project';
  import Wrench from '~icons/fa6-solid/wrench';

  interface Props {
    icon?: string;
    class?: string;
  }

  let { icon, class: className = 'h-5 w-5' }: Props = $props();
</script>

{#if icon === 'terminal'}
  <Terminal class={className} />
{:else if icon === 'folder'}
  <Folder class={className} />
{:else if icon === 'check-square'}
  <CheckSquare class={className} />
{:else if icon === 'code'}
  <Code class={className} />
{:else if icon === 'lock'}
  <Lock class={className} />
{:else if icon === 'rocket'}
  <Rocket class={className} />
{:else if icon === 'file-text'}
  <FileText class={className} />
{:else if icon === 'settings'}
  <Settings class={className} />
{:else if icon === 'database'}
  <Database class={className} />
{:else if icon === 'container' || icon === 'blocks'}
  <Boxes class={className} />
{:else if icon === 'cloud'}
  <Cloud class={className} />
{:else if icon === 'globe'}
  <Globe class={className} />
{:else if icon === 'github'}
  <FolderGit2 class={className} />
{:else if icon === 'robot'}
  <Bot class={className} />
{:else if icon === 'wrench'}
  <Wrench class={className} />
{:else if icon === 'file-code'}
  <FileCode class={className} />
{:else if icon === 'sparkles'}
  <Sparkles class={className} />
{:else if icon === 'workflow'}
  <Workflow class={className} />
{:else if icon === 'message-square'}
  <MessageSquare class={className} />
{:else if icon === 'history'}
  <History class={className} />
{:else if icon === 'hard-drive'}
  <HardDrive class={className} />
{:else if icon === 'scan-search'}
  <ScanSearch class={className} />
{:else if icon === 'variable'}
  <Variable class={className} />
{:else}
  <Home class={className} />
{/if}
