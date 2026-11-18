<!--
	Terminal Settings - Terminal configuration and preferences
-->

<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import SettingsSection from './shared/SettingsSection.svelte';
  import SettingsSwitchRow from './shared/SettingsSwitchRow.svelte';
  import { settingsService } from '../services/settingsService';
  import type { TerminalSettings } from '../types';

  interface Props {
    settings: TerminalSettings | null;
    onUpdate: (updates: Partial<TerminalSettings>) => void;
  }

  let { settings, onUpdate }: Props = $props();

  if (!settings) {
    settings = {
      fontFamily: 'monaco',
      fontSize: 14,
      lineHeight: 1.2,
      cursorStyle: 'block',
      cursorBlink: true,
      scrollback: 1000,
      bellStyle: 'none',
      rightClickSelectsWord: false,
      selectionMode: 'normal',
      shellIntegration: true,
      commandHistory: true,
      commandSuggestions: true,
      theme: {
        name: 'default',
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff',
        selection: '#264f78',
        colors: {
          black: '#000000',
          red: '#cd3131',
          green: '#0dbc79',
          yellow: '#e5e510',
          blue: '#2472c8',
          magenta: '#bc3fbc',
          cyan: '#11a8cd',
          white: '#e5e5e5',
          brightBlack: '#666666',
          brightRed: '#f14c4c',
          brightGreen: '#23d18b',
          brightYellow: '#f5f543',
          brightBlue: '#3b8eea',
          brightMagenta: '#d670d6',
          brightCyan: '#29b8db',
          brightWhite: '#e5e5e5',
        },
      },
      encoding: 'utf-8',
      locale: 'en_US.UTF-8',
    };
  }

  const availableFonts = settingsService.getAvailableFonts();
  const availableThemes = settingsService.getAvailableTerminalThemes();
</script>

<div class="space-y-6">
  <!-- Appearance -->
  <SettingsSection title="Appearance" description="Configure terminal visual appearance">
    <div class="space-y-2">
      <Label for="terminalFontFamily">Font Family</Label>
      <Select
        defaultValue={settings.fontFamily}
        options={availableFonts.map((f) => ({
          value: f.id,
          label: `${f.name} (${f.category})`,
        }))}
        onSelect={(value) => onUpdate({ fontFamily: value })}
        placeholder="Select font"
      />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-2">
        <Label for="terminalFontSize">Font Size</Label>
        <Input
          id="terminalFontSize"
          type="number"
          min="8"
          max="72"
          bind:value={settings.fontSize}
          onchange={() => onUpdate({ fontSize: parseInt(String(settings.fontSize)) || 14 })}
        />
      </div>

      <div class="space-y-2">
        <Label for="terminalLineHeight">Line Height</Label>
        <Input
          id="terminalLineHeight"
          type="number"
          min="1"
          max="3"
          step="0.1"
          bind:value={settings.lineHeight}
          onchange={() =>
            onUpdate({
              lineHeight: parseFloat(String(settings.lineHeight)) || 1.2,
            })}
        />
      </div>
    </div>

    <div class="space-y-2">
      <Label for="terminalTheme">Terminal Theme</Label>
      <Select
        defaultValue={settings.theme.name}
        options={availableThemes.map((t) => ({ value: t.id, label: t.name }))}
        onSelect={(value) => {
          const theme = availableThemes.find((t) => t.id === value);
          if (theme) {
            onUpdate({
              theme: {
                name: theme.name,
                background: theme.colors.background,
                foreground: theme.colors.foreground,
                cursor: theme.colors.cursor,
                selection: theme.colors.selection || '#264f78',
                colors: settings.theme.colors,
              },
            });
          }
        }}
        placeholder="Select theme"
      />
    </div>
  </SettingsSection>

  <!-- Cursor -->
  <SettingsSection title="Cursor" description="Configure terminal cursor appearance">
    <div class="space-y-2">
      <Label for="cursorStyle">Cursor Style</Label>
      <Select
        defaultValue={settings.cursorStyle}
        options={[
          { value: 'block', label: 'Block' },
          { value: 'underline', label: 'Underline' },
          { value: 'line', label: 'Line' },
        ]}
        onSelect={(value) => onUpdate({ cursorStyle: value as 'block' | 'underline' | 'line' })}
      />
    </div>

    <SettingsSwitchRow
      id="cursorBlink"
      label="Cursor Blink"
      description="Animate cursor blinking"
      checked={settings.cursorBlink}
      onToggle={() => onUpdate({ cursorBlink: !settings.cursorBlink })}
    />
  </SettingsSection>

  <!-- Behavior -->
  <SettingsSection title="Behavior" description="Configure terminal behavior and interactions">
    <div class="space-y-2">
      <Label for="scrollback">Scrollback Lines</Label>
      <Input
        id="scrollback"
        type="number"
        min="100"
        max="10000"
        bind:value={settings.scrollback}
        onchange={() =>
          onUpdate({
            scrollback: parseInt(String(settings.scrollback)) || 1000,
          })}
      />
    </div>

    <div class="space-y-2">
      <Label for="bellStyle">Bell Style</Label>
      <Select
        defaultValue={settings.bellStyle}
        options={[
          { value: 'none', label: 'None' },
          { value: 'visual', label: 'Visual' },
          { value: 'sound', label: 'Sound' },
        ]}
        onSelect={(value) => onUpdate({ bellStyle: value as 'none' | 'visual' | 'sound' })}
      />
    </div>

    <SettingsSwitchRow
      id="rightClickSelectsWord"
      label="Right Click Selects Word"
      description="Right click to select word"
      checked={settings.rightClickSelectsWord}
      onToggle={() => onUpdate({ rightClickSelectsWord: !settings.rightClickSelectsWord })}
    />

    <div class="space-y-2">
      <Label for="selectionMode">Selection Mode</Label>
      <Select
        defaultValue={settings.selectionMode}
        options={['normal', 'column']}
        onSelect={(value) => onUpdate({ selectionMode: value as 'normal' | 'column' })}
        placeholder="Select mode"
      />
    </div>
  </SettingsSection>

  <!-- Shell Integration -->
  <SettingsSection title="Shell Integration" description="Configure shell integration features">
    <SettingsSwitchRow
      id="shellIntegration"
      label="Shell Integration"
      description="Enable shell integration features"
      checked={settings.shellIntegration}
      onToggle={() => onUpdate({ shellIntegration: !settings.shellIntegration })}
    />

    <SettingsSwitchRow
      id="commandHistory"
      label="Command History"
      description="Save command history"
      checked={settings.commandHistory}
      onToggle={() => onUpdate({ commandHistory: !settings.commandHistory })}
    />

    <SettingsSwitchRow
      id="commandSuggestions"
      label="Command Suggestions"
      description="Show command suggestions"
      checked={settings.commandSuggestions}
      onToggle={() => onUpdate({ commandSuggestions: !settings.commandSuggestions })}
    />
  </SettingsSection>

  <!-- Advanced -->
  <SettingsSection title="Advanced" description="Advanced terminal configuration">
    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-2">
        <Label for="encoding">Encoding</Label>
        <Input
          id="encoding"
          bind:value={settings.encoding}
          onchange={() => onUpdate({ encoding: settings.encoding })}
        />
      </div>

      <div class="space-y-2">
        <Label for="locale">Locale</Label>
        <Input
          id="locale"
          bind:value={settings.locale}
          onchange={() => onUpdate({ locale: settings.locale })}
        />
      </div>
    </div>
  </SettingsSection>
</div>
