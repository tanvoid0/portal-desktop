<script lang="ts">
  import { Button } from './button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from './dropdown-menu';
  import { themeStore, currentTheme } from '$lib/domains/shared';
  import Sun from '~icons/fa6-solid/sun';
  import Moon from '~icons/fa6-solid/moon';
  import Monitor from '~icons/fa6-solid/desktop';

  let isOpen = false;

  function setTheme(theme: 'light' | 'dark' | 'system') {
    themeStore.setTheme(theme);
    isOpen = false;
  }
</script>

<DropdownMenu bind:open={isOpen}>
  <DropdownMenuTrigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="sm" class="h-8 w-8 px-0">
        {#if $currentTheme === 'system'}
          <Monitor class="h-4 w-4" />
        {:else if $currentTheme === 'light'}
          <Sun class="h-4 w-4" />
        {:else}
          <Moon class="h-4 w-4" />
        {/if}
        <span class="sr-only">Toggle theme</span>
      </Button>
    {/snippet}
  </DropdownMenuTrigger>
  <DropdownMenuContent align="end">
    <DropdownMenuItem onclick={() => setTheme('light')}>
      <Sun class="mr-2 h-4 w-4" />
      <span>Light</span>
    </DropdownMenuItem>
    <DropdownMenuItem onclick={() => setTheme('dark')}>
      <Moon class="mr-2 h-4 w-4" />
      <span>Dark</span>
    </DropdownMenuItem>
    <DropdownMenuItem onclick={() => setTheme('system')}>
      <Monitor class="mr-2 h-4 w-4" />
      <span>System</span>
    </DropdownMenuItem>
  </DropdownMenuContent>
</DropdownMenu>
