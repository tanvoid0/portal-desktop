<script lang="ts">
  import { terminalStore, terminalActions, activeTab, tabCount } from '../stores/terminalStore';
  import { Plus, X, ChevronDown } from 'lucide-svelte';
  import { TerminalService } from '../services/terminalService';
  import { Button } from '@/lib/components/ui/button';
  import Select from '@/lib/components/ui/select.svelte';
  import { onMount } from 'svelte';

  // Props
  export let onNewTab: ((profileName?: string) => void) | null = null;
  export let showNewTabButton = true;
  export let closable = true;
  export let showProfileSelector = true;

  // Terminal profile state
  let availableProfiles: any[] = [];
  let selectedProfile: string = '';
  let systemInfo: any = null;

  // Reactive stores
  $: tabs = $terminalStore.tabs;
  $: activeTabId = $terminalStore.activeTabId;
  $: currentActiveTab = $activeTab;

  function handleTabClick(tabId: string) {
    terminalActions.setActiveTab(tabId);
  }

  function handleCloseTab(tabId: string, event: Event) {
    event.stopPropagation();
    terminalActions.closeTab(tabId);
  }

  function handleNewTab() {
    console.log('TabBar: New tab button clicked');
    if (onNewTab) {
      console.log('TabBar: Calling onNewTab callback');
      onNewTab();
    } else {
      console.log('TabBar: No onNewTab callback provided');
    }
  }

  async function loadSystemInfo() {
    try {
      console.log('TabBar: Loading system info for profile selector');
      systemInfo = await TerminalService.getSystemInfo();
      
      if (systemInfo?.terminal_profiles) {
        // Extract available profiles from system info
        const profiles: any[] = [];
        
        // Add available shells
        if (systemInfo.terminal_profiles.available_shells) {
          Object.entries(systemInfo.terminal_profiles.available_shells).forEach(([name, info]: [string, any]) => {
            profiles.push({
              name,
              command: info.command || name,
              icon: getProfileIcon(name),
              category: 'shell'
            });
          });
        }
        
        // Add Windows Terminal profiles if available
        if (systemInfo.terminal_profiles.windows_terminal) {
          systemInfo.terminal_profiles.windows_terminal.forEach((profile: any) => {
            profiles.push({
              name: profile.name,
              command: profile.commandline || profile.name,
              icon: getProfileIcon(profile.name),
              category: 'windows_terminal'
            });
          });
        }
        
        // Remove duplicates based on name
        const uniqueProfiles = profiles.filter((profile, index, self) => 
          index === self.findIndex(p => p.name === profile.name)
        );
        
        availableProfiles = uniqueProfiles;
        console.log('TabBar: Available profiles:', availableProfiles);
        
        // Set default profile
        if (availableProfiles.length > 0) {
          selectedProfile = availableProfiles[0].name;
        }
      } else {
        console.log('TabBar: No terminal profiles found in system info');
        availableProfiles = [];
      }
    } catch (error) {
      console.error('TabBar: Failed to load system info:', error);
      availableProfiles = [];
    }
  }

  function getProfileIcon(profileName: string): string {
    const name = profileName.toLowerCase();
    if (name.includes('cmd')) return '🖥️';
    if (name.includes('powershell') || name.includes('pwsh')) return '💙';
    if (name.includes('bash')) return '🐧';
    if (name.includes('zsh')) return '⚡';
    if (name.includes('fish')) return '🐠';
    if (name.includes('wsl')) return '🐧';
    return '💻';
  }

  function handleProfileChange(value: string) {
    selectedProfile = value;
    console.log('TabBar: Profile changed to:', selectedProfile);
  }

  function createNewTabWithProfile() {
    console.log('TabBar: Creating new tab with profile:', selectedProfile);
    
    // Find the profile and extract just the raw command
    const profile = availableProfiles.find(p => p.name === selectedProfile);
    
    if (profile) {
      console.log('TabBar: Using raw shell command:', profile.command);
      if (onNewTab) {
        onNewTab(profile.command); // Pass just the raw shell command
      }
    } else {
      console.warn('TabBar: Profile not found:', selectedProfile);
    }
  }

  onMount(() => {
    if (showProfileSelector) {
      loadSystemInfo();
    }
  });

  function getTabStatusColor(tab: any) {
    switch (tab.status) {
      case 'active':
        return 'border-blue-500';
      case 'loading':
        return 'border-yellow-500';
      case 'error':
        return 'border-red-500';
      default:
        return 'border-transparent';
    }
  }

  function getTabIcon(tab: any) {
    if (tab.icon) return tab.icon;
    
    switch (tab.type) {
      case 'terminal':
        return '💻';
      case 'editor':
        return '📝';
      case 'file':
        return '📄';
      default:
        return '📋';
    }
  }
</script>

<div class="tab-bar bg-gray-800 border-b border-gray-700 flex items-center min-h-[40px]">
  <!-- Tab List -->
  <div class="flex-1 flex overflow-x-auto">
    {#each tabs as tab (tab.id)}
      <div
        onclick={() => handleTabClick(tab.id)}
        class="tab flex items-center space-x-2 px-4 py-2 text-sm border-r border-gray-700 border-b-2 transition-colors min-w-0 cursor-pointer {activeTabId === tab.id ? 'bg-gray-700 text-white' : 'bg-gray-800 text-gray-300 hover:bg-gray-700'} {getTabStatusColor(tab)}"
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && handleTabClick(tab.id)}
        title={tab.title}
      >
        <span class="text-xs">{getTabIcon(tab)}</span>
        <span class="truncate max-w-32">{tab.title}</span>
        {#if closable && tab.closable !== false && tabs.length > 1}
          <Button
            variant="ghost"
            size="sm"
            onclick={(e) => handleCloseTab(tab.id, e)}
            class="ml-1 p-1 h-auto"
            type="button"
            aria-label="Close tab"
          >
            <X size={12} />
          </Button>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Profile Selector -->
  {#if showProfileSelector}
    <div class="flex items-center space-x-2 px-2">
      <Select
        onSelect={handleProfileChange}
        class="bg-gray-700 text-gray-200 text-xs"
        options={availableProfiles.length === 0 ? [{ value: '', label: 'Loading profiles...' }] : availableProfiles.map(profile => ({ value: profile.name, label: `${profile.icon} ${profile.name}` }))}
        defaultValue={selectedProfile}
      />
    </div>
  {/if}

  <!-- New Tab Button -->
  {#if showNewTabButton && onNewTab}
    <Button
      variant="ghost"
      size="sm"
      onclick={createNewTabWithProfile}
      class="p-2 text-gray-400 hover:text-white hover:bg-gray-700"
      type="button"
      aria-label="New tab"
      title="New Tab with selected profile (Ctrl+T)"
    >
      <Plus size={16} />
    </Button>
  {/if}
</div>

<style>
  .tab-bar {
    scrollbar-width: thin;
    scrollbar-color: #4a5568 #2d3748;
  }

  .tab-bar::-webkit-scrollbar {
    height: 4px;
  }

  .tab-bar::-webkit-scrollbar-track {
    background: #2d3748;
  }

  .tab-bar::-webkit-scrollbar-thumb {
    background: #4a5568;
    border-radius: 2px;
  }

  .tab-bar::-webkit-scrollbar-thumb:hover {
    background: #718096;
  }

  .tab {
    white-space: nowrap;
  }
</style>
