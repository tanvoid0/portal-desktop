<!--
	Editor Settings - Code editor preferences and configuration
-->

<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import { Input } from '$lib/components/ui/input';
  import Select from '$lib/components/ui/select.svelte';
  import SettingsSection from './shared/SettingsSection.svelte';
  import SettingsSwitchRow from './shared/SettingsSwitchRow.svelte';
  import { Separator } from '$lib/components/ui/separator';
  import { settingsService } from '../services/settingsService';
  import type { EditorSettings } from '../types';

  interface Props {
    settings: EditorSettings | null;
    onUpdate: (updates: Partial<EditorSettings>) => void;
  }

  let { settings, onUpdate }: Props = $props();

  if (!settings) {
    settings = {
      fontFamily: 'monaco',
      fontSize: 14,
      lineHeight: 1.5,
      tabSize: 2,
      insertSpaces: true,
      wordWrap: true,
      showLineNumbers: true,
      showMinimap: false,
      showWhitespace: false,
      syntaxHighlighting: true,
      bracketMatching: true,
      autoIndent: true,
      autoComplete: true,
      suggestions: true,
      parameterHints: true,
      editorTheme: 'default',
      terminalTheme: 'default',
      keybindings: {},
    };
  }

  const availableFonts = settingsService.getAvailableFonts();
</script>

<div class="space-y-6">
  <!-- Font & Display -->
  <SettingsSection title="Font & Display" description="Configure editor font and visual appearance">
    <div class="space-y-2">
      <Label for="fontFamily">Font Family</Label>
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
        <Label for="fontSize">Font Size</Label>
        <Input
          id="fontSize"
          type="number"
          min="8"
          max="72"
          bind:value={settings.fontSize}
          onchange={() => onUpdate({ fontSize: parseInt(String(settings.fontSize)) || 14 })}
        />
      </div>

      <div class="space-y-2">
        <Label for="lineHeight">Line Height</Label>
        <Input
          id="lineHeight"
          type="number"
          min="1"
          max="3"
          step="0.1"
          bind:value={settings.lineHeight}
          onchange={() =>
            onUpdate({
              lineHeight: parseFloat(String(settings.lineHeight)) || 1.5,
            })}
        />
      </div>
    </div>
  </SettingsSection>

  <!-- Indentation -->
  <SettingsSection title="Indentation" description="Configure how code is indented">
    <div class="grid grid-cols-2 gap-4">
      <div class="space-y-2">
        <Label for="tabSize">Tab Size</Label>
        <Input
          id="tabSize"
          type="number"
          min="1"
          max="8"
          bind:value={settings.tabSize}
          onchange={() => onUpdate({ tabSize: parseInt(String(settings.tabSize)) || 2 })}
        />
      </div>

      <SettingsSwitchRow
        id="insertSpaces"
        label="Insert Spaces"
        description="Use spaces instead of tabs"
        checked={settings.insertSpaces}
        onToggle={() => onUpdate({ insertSpaces: !settings.insertSpaces })}
      />
    </div>

    <SettingsSwitchRow
      id="autoIndent"
      label="Auto Indent"
      description="Automatically indent code"
      checked={settings.autoIndent}
      onToggle={() => onUpdate({ autoIndent: !settings.autoIndent })}
    />
  </SettingsSection>

  <!-- Visual Features -->
  <SettingsSection title="Visual Features" description="Configure editor visual elements">
    <SettingsSwitchRow
      id="wordWrap"
      label="Word Wrap"
      description="Wrap long lines"
      checked={settings.wordWrap}
      onToggle={() => onUpdate({ wordWrap: !settings.wordWrap })}
    />

    <SettingsSwitchRow
      id="showLineNumbers"
      label="Show Line Numbers"
      description="Display line numbers"
      checked={settings.showLineNumbers}
      onToggle={() => onUpdate({ showLineNumbers: !settings.showLineNumbers })}
    />

    <SettingsSwitchRow
      id="showMinimap"
      label="Show Minimap"
      description="Display code minimap"
      checked={settings.showMinimap}
      onToggle={() => onUpdate({ showMinimap: !settings.showMinimap })}
    />

    <SettingsSwitchRow
      id="showWhitespace"
      label="Show Whitespace"
      description="Display whitespace characters"
      checked={settings.showWhitespace}
      onToggle={() => onUpdate({ showWhitespace: !settings.showWhitespace })}
    />
  </SettingsSection>

  <!-- Code Features -->
  <SettingsSection title="Code Features" description="Configure code editing features">
    <SettingsSwitchRow
      id="syntaxHighlighting"
      label="Syntax Highlighting"
      description="Color code syntax"
      checked={settings.syntaxHighlighting}
      onToggle={() => onUpdate({ syntaxHighlighting: !settings.syntaxHighlighting })}
    />

    <SettingsSwitchRow
      id="bracketMatching"
      label="Bracket Matching"
      description="Highlight matching brackets"
      checked={settings.bracketMatching}
      onToggle={() => onUpdate({ bracketMatching: !settings.bracketMatching })}
    />

    <Separator />

    <SettingsSwitchRow
      id="autoComplete"
      label="Auto Complete"
      description="Enable auto-completion"
      checked={settings.autoComplete}
      onToggle={() => onUpdate({ autoComplete: !settings.autoComplete })}
    />

    <SettingsSwitchRow
      id="suggestions"
      label="Suggestions"
      description="Show code suggestions"
      checked={settings.suggestions}
      onToggle={() => onUpdate({ suggestions: !settings.suggestions })}
    />

    <SettingsSwitchRow
      id="parameterHints"
      label="Parameter Hints"
      description="Show function parameter hints"
      checked={settings.parameterHints}
      onToggle={() => onUpdate({ parameterHints: !settings.parameterHints })}
    />
  </SettingsSection>
</div>
