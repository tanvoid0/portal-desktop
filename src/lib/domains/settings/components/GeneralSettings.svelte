<!--
	General Settings - Application preferences and general configuration
-->

<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import { Separator } from '$lib/components/ui/separator';
  import { settingsService } from '../services/settingsService';
  import { mergeSettings } from '../utils/mergeSettings';
  import type { AppSettings } from '../types';
  import SettingsSection from './shared/SettingsSection.svelte';
  import SettingsSwitchRow from './shared/SettingsSwitchRow.svelte';

  interface Props {
    settings: AppSettings | null;
    onUpdate: (updates: Partial<AppSettings>) => void;
  }

  let { settings: settingsProp, onUpdate }: Props = $props();

  const settings = $derived(mergeSettings(settingsProp));

  // Reactive local state for inputs
  let timezone = $state('');
  let dateFormat = $state('YYYY-MM-DD');

  // Update local state when settings change
  $effect(() => {
    timezone = settings.timezone || '';
    dateFormat = settings.dateFormat || 'YYYY-MM-DD';
  });

  const availableLanguages = settingsService.getAvailableLanguages();

  type Group = 'startupBehavior' | 'notifications' | 'privacy' | 'updates';
  type Row = { group: Group; key: string; label: string; description?: string };

  const startupRows: Row[] = [
    {
      group: 'startupBehavior',
      key: 'openLastSession',
      label: 'Open Last Session',
      description: 'Automatically open the last session on startup',
    },
    {
      group: 'startupBehavior',
      key: 'restoreWindows',
      label: 'Restore Windows',
      description: 'Restore window positions and sizes',
    },
    {
      group: 'startupBehavior',
      key: 'showWelcomeScreen',
      label: 'Show Welcome Screen',
      description: 'Display welcome screen on first launch',
    },
    {
      group: 'startupBehavior',
      key: 'minimizeToTray',
      label: 'Minimize to Tray',
      description: 'Minimize to system tray instead of taskbar',
    },
  ];
  const privacyRows: Row[] = [
    {
      group: 'privacy',
      key: 'crashReports',
      label: 'Send Crash Reports',
      description: 'Automatically send crash reports to help improve the app',
    },
    {
      group: 'privacy',
      key: 'analytics',
      label: 'Analytics',
      description: 'Share usage analytics to improve features',
    },
  ];

  function flag(group: Group, key: string): boolean {
    return (settings[group] as unknown as Record<string, boolean> | undefined)?.[key] ?? false;
  }

  function toggle(group: Group, key: string) {
    onUpdate({ [group]: { ...settings[group], [key]: !flag(group, key) } });
  }
</script>

{#snippet rows(list: Row[])}
  {#each list as r (r.key)}
    <SettingsSwitchRow
      id={r.key}
      label={r.label}
      description={r.description}
      checked={flag(r.group, r.key)}
      onToggle={() => toggle(r.group, r.key)}
    />
  {/each}
{/snippet}

<div class="space-y-6">
  <SettingsSection title="Appearance" description="Configure the look and feel of the application">
    <div class="space-y-2">
      <Label for="language">Language</Label>
      <Select
        defaultValue={settings.language}
        options={availableLanguages.map((l) => ({
          value: l.id,
          label: `${l.name} (${l.native})`,
        }))}
        onSelect={(value) => onUpdate({ language: value })}
        placeholder="Select language"
      />
    </div>

    <div class="space-y-2">
      <Label for="timezone">Timezone</Label>
      <Input id="timezone" bind:value={timezone} onchange={() => onUpdate({ timezone })} />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-2">
        <Label for="dateFormat">Date Format</Label>
        <Input
          id="dateFormat"
          bind:value={dateFormat}
          onchange={() => onUpdate({ dateFormat })}
          placeholder="YYYY-MM-DD"
        />
      </div>

      <div class="space-y-2">
        <Label for="timeFormat">Time Format</Label>
        <Select
          defaultValue={settings.timeFormat}
          options={['12h', '24h']}
          onSelect={(value) => onUpdate({ timeFormat: value as '12h' | '24h' })}
          placeholder="Select format"
        />
      </div>
    </div>
  </SettingsSection>

  <SettingsSection title="Startup Behavior" description="Configure how the application starts">
    {@render rows(startupRows)}
  </SettingsSection>

  <SettingsSection title="Notifications" description="Configure notification preferences">
    <SettingsSwitchRow
      id="notificationsEnabled"
      label="Enable Notifications"
      description="Receive notifications from the application"
      checked={flag('notifications', 'enabled')}
      onToggle={() => toggle('notifications', 'enabled')}
    />

    <Separator />

    <div class="space-y-3">
      {#each [{ key: 'desktopNotifications', label: 'Desktop Notifications' }, { key: 'soundEnabled', label: 'Sound Notifications' }] as r (r.key)}
        <SettingsSwitchRow
          id={r.key}
          label={r.label}
          checked={flag('notifications', r.key)}
          disabled={!flag('notifications', 'enabled')}
          onToggle={() => toggle('notifications', r.key)}
        />
      {/each}
    </div>
  </SettingsSection>

  <SettingsSection title="Privacy" description="Configure privacy and data collection settings">
    {@render rows(privacyRows)}
  </SettingsSection>

  <SettingsSection title="Updates" description="Configure automatic update behavior">
    <SettingsSwitchRow
      id="autoCheck"
      label="Auto-check for Updates"
      description="Automatically check for updates"
      checked={flag('updates', 'autoCheck')}
      onToggle={() => toggle('updates', 'autoCheck')}
    />
    <SettingsSwitchRow
      id="notifyOnUpdate"
      label="Notify on Update"
      description="Show notification when updates are available"
      checked={flag('updates', 'notifyOnUpdate')}
      disabled={!flag('updates', 'autoCheck')}
      onToggle={() => toggle('updates', 'notifyOnUpdate')}
    />

    <div class="space-y-2">
      <Label for="updateChannel">Update Channel</Label>
      <Select
        defaultValue={settings.updates?.channel || 'stable'}
        options={['stable', 'beta', 'alpha']}
        onSelect={(value) =>
          onUpdate({
            updates: {
              ...settings.updates,
              channel: value as 'stable' | 'beta' | 'alpha',
            },
          })}
        placeholder="Select channel"
      />
    </div>
  </SettingsSection>
</div>
