<!-- Static explainer card for the autonomous action system. -->
<script lang="ts">
  import Shield from '~icons/fa6-solid/shield';
  import SettingsSection from '../shared/SettingsSection.svelte';
</script>

<SettingsSection
  title="How Autonomy Works"
  icon={Shield}
  contentClass="space-y-3 text-sm text-muted-foreground"
>
  <p>
    The autonomous action system learns from your workflow patterns and gradually increases its
    autonomy based on successful actions.
  </p>
  <ul class="ml-2 list-inside list-disc space-y-1">
    <li><strong>Safe actions:</strong> Read-only operations, suggestions, UI updates</li>
    <li><strong>Low-risk actions:</strong> Common commands like npm install, git status</li>
    <li><strong>Medium-risk actions:</strong> Configuration changes, file creation</li>
    <li>
      <strong>High-risk actions:</strong> Always require approval (file deletion, system changes)
    </li>
  </ul>
  <p class="pt-2">
    The system tracks approval rates and success rates to determine when actions can be executed
    autonomously. You can always review and adjust these settings.
  </p>
</SettingsSection>
