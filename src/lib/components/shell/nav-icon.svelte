<script lang="ts">
  import {
    Bot,
    Boxes,
    CheckSquare,
    Cloud,
    Code,
    Database,
    FileCode,
    FileText,
    Folder,
    FolderGit2,
    Globe,
    HardDrive,
    History,
    Home,
    Lock,
    MessageSquare,
    Rocket,
    Settings,
    Sparkles,
    Terminal,
    Variable,
    Workflow,
    Wrench,
  } from "@lucide/svelte";

  interface Props {
    icon?: string;
    class?: string;
  }

  let { icon, class: className = "h-5 w-5" }: Props = $props();
</script>
{#if icon === "terminal"}
  <Terminal class={className} />
{:else if icon === "folder"}
  <Folder class={className} />
{:else if icon === "check-square"}
  <CheckSquare class={className} />
{:else if icon === "code"}
  <Code class={className} />
{:else if icon === "lock"}
  <Lock class={className} />
{:else if icon === "rocket"}
  <Rocket class={className} />
{:else if icon === "file-text"}
  <FileText class={className} />
{:else if icon === "settings"}
  <Settings class={className} />
{:else if icon === "database"}
  <Database class={className} />
{:else if icon === "container" || icon === "blocks"}
  <Boxes class={className} />
{:else if icon === "cloud"}
  <Cloud class={className} />
{:else if icon === "globe"}
  <Globe class={className} />
{:else if icon === "github"}
  <FolderGit2 class={className} />
{:else if icon === "robot"}
  <Bot class={className} />
{:else if icon === "wrench"}
  <Wrench class={className} />
{:else if icon === "file-code"}
  <FileCode class={className} />
{:else if icon === "sparkles"}
  <Sparkles class={className} />
{:else if icon === "workflow"}
  <Workflow class={className} />
{:else if icon === "message-square"}
  <MessageSquare class={className} />
{:else if icon === "history"}
  <History class={className} />
{:else if icon === "hard-drive"}
  <HardDrive class={className} />
{:else if icon === "variable"}
  <Variable class={className} />
{:else}
  <Home class={className} />
{/if}
