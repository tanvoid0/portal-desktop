<!--
	Learning Settings - ML Learning System Configuration
-->

<script lang="ts">
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { onMount } from 'svelte';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Separator } from '$lib/components/ui/separator';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import Select from '$lib/components/ui/select.svelte';
  import {
    learningService,
    memoryService,
    type MLIntensity,
    type MemoryStats,
    type CleanupPreview,
  } from '$lib/domains/learning';
  import { logger } from '$lib/domains/shared/services/logger';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { goto } from '$app/navigation';
  import Brain from '~icons/fa6-solid/brain';
  import Activity from '~icons/fa6-solid/wave-square';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import Zap from '~icons/fa6-solid/bolt';
  import Settings from '~icons/fa6-solid/gear';
  import CheckCircle2 from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import SettingsSection from './shared/SettingsSection.svelte';
  import LearningMemoryUsage from './learning/LearningMemoryUsage.svelte';
  import LearningDashboard from './learning/LearningDashboard.svelte';
  import { intensityOptions, intensityDescriptions } from './learning/intensity';

  const log = logger.createScoped('LearningSettings');

  let mlEnabled = $state(true);
  let mlIntensity = $state<MLIntensity | null>(null); // Start as null, will be set after load
  let memoryStats = $state<MemoryStats | null>(null);
  let cleanupPreview = $state<CleanupPreview | null>(null);
  let isLoading = $state(false);
  let isRefreshing = $state(false);
  let lastUpdate = $state<Date | null>(null);

  const intensityIcons = { fast: Zap, light: Sparkles, medium: Brain, deep: Activity };

  onMount(loadSettings);

  async function loadSettings() {
    isLoading = true;
    try {
      [mlEnabled, mlIntensity, memoryStats, cleanupPreview] = await Promise.all([
        learningService.getMLEnabled(),
        learningService.getMLIntensity(),
        memoryService.getStats().catch(() => null),
        memoryService.getCleanupPreview().catch(() => null),
      ]);
      lastUpdate = new Date();
      log.info('Learning settings loaded');
    } catch (error) {
      log.error('Failed to load learning settings', error);
      toast.error('Failed to load learning settings');
    } finally {
      isLoading = false;
    }
  }

  async function handleToggleEnabled() {
    try {
      await learningService.setMLEnabled(mlEnabled);
      toast.success(`ML Learning ${mlEnabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      log.error('Failed to toggle ML enabled state', error);
      toast.error('Failed to update ML enabled state');
      mlEnabled = !mlEnabled; // Revert
    }
  }

  async function handleIntensityChange(newIntensity: MLIntensity) {
    try {
      mlIntensity = newIntensity;
      await learningService.setMLIntensity(newIntensity);
      toast.success(
        `ML Intensity set to ${intensityOptions.find((o) => o.value === newIntensity)?.label}`
      );
    } catch (error) {
      log.error('Failed to set ML intensity', error);
      toast.error('Failed to update ML intensity');
      await loadSettings(); // Reload to revert
    }
  }

  async function handleCleanup() {
    if (!cleanupPreview) return;

    const totalToDelete = cleanupPreview.events_to_delete + cleanupPreview.patterns_to_delete;
    if (totalToDelete === 0) {
      toast.info('No data to clean up');
      return;
    }

    const confirmed = await confirmAction(
      `This will delete ${totalToDelete} items. Continue?`,
      'Clean up memory'
    );
    if (!confirmed) return;

    isRefreshing = true;
    try {
      await memoryService.cleanup();
      toast.success('Memory cleanup completed');
      await refreshStats();
    } catch (error) {
      log.error('Failed to cleanup memory', error);
      toast.error('Failed to cleanup memory');
    } finally {
      isRefreshing = false;
    }
  }

  async function refreshStats() {
    isRefreshing = true;
    try {
      [memoryStats, cleanupPreview] = await Promise.all([
        memoryService.getStats().catch(() => null),
        memoryService.getCleanupPreview().catch(() => null),
      ]);
      lastUpdate = new Date();
    } catch (error) {
      log.error('Failed to refresh stats', error);
    } finally {
      isRefreshing = false;
    }
  }
</script>

<div class="space-y-6">
  <SettingsSection
    title="AI Provider Configuration"
    description="AI provider settings have been moved to the dedicated AI page"
    icon={Sparkles}
    contentClass=""
  >
    <Button onclick={() => goto('/settings/ai')}>
      <Sparkles class="mr-2 h-4 w-4" />
      Go to AI Providers
    </Button>
  </SettingsSection>

  <Separator />

  <SettingsSection
    title="Learning System Status"
    description="Current state and activity of the ML learning system"
    icon={Activity}
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Label for="ml-enabled" class="text-base font-medium">ML Learning Enabled</Label>
        {#if mlEnabled}
          <Badge variant="default" class="bg-green-500">
            <CheckCircle2 class="mr-1 h-3 w-3" />
            Active
          </Badge>
        {:else}
          <Badge variant="secondary">
            <XCircle class="mr-1 h-3 w-3" />
            Disabled
          </Badge>
        {/if}
      </div>
      <Switch
        id="ml-enabled"
        checked={mlEnabled}
        onCheckedChange={handleToggleEnabled}
        disabled={isLoading}
      />
    </div>

    <Separator />

    <div class="grid grid-cols-2 gap-4">
      <div>
        <Label class="text-sm text-muted-foreground">Current Intensity</Label>
        {#if mlIntensity === null}
          <Skeleton class="mt-1 h-6 rounded bg-muted" />
        {:else}
          {@const Icon = intensityIcons[mlIntensity]}
          <div class="mt-1 flex items-center gap-2">
            <Icon class="h-4 w-4" />
            <span class="font-medium capitalize">{mlIntensity}</span>
          </div>
        {/if}
      </div>
      <div>
        <Label class="text-sm text-muted-foreground">Initialized</Label>
        <div class="mt-1">
          <Badge variant="outline">Yes</Badge>
        </div>
      </div>
    </div>

    {#if lastUpdate}
      <div class="text-xs text-muted-foreground">
        Last updated: {lastUpdate.toLocaleTimeString()}
      </div>
    {/if}
  </SettingsSection>

  <SettingsSection
    title="Resource Usage Control"
    description="Balance between learning intelligence and system performance"
    icon={Settings}
  >
    <div class="space-y-2">
      <Label>ML Intensity Level</Label>
      {#if mlIntensity !== null}
        <Select
          options={intensityOptions.map((o) => ({
            value: o.value,
            label: `${o.label} - ${o.description}`,
          }))}
          defaultValue={mlIntensity}
          onSelect={(value) => value && handleIntensityChange(value as MLIntensity)}
          disabled={!mlEnabled || isLoading}
        />
      {:else}
        <Skeleton class="h-10 border border-input bg-muted" />
      {/if}
      {#if mlIntensity !== null}
        <p class="text-sm text-muted-foreground">
          {intensityDescriptions[mlIntensity]}
        </p>
      {/if}
    </div>

    {#if !mlEnabled}
      <div class="rounded-md bg-muted p-3 text-sm text-muted-foreground">
        Enable ML Learning above to adjust intensity settings.
      </div>
    {/if}
  </SettingsSection>

  <LearningMemoryUsage
    {memoryStats}
    {cleanupPreview}
    {isLoading}
    {isRefreshing}
    onCleanup={handleCleanup}
    onRefresh={refreshStats}
  />

  <LearningDashboard />
</div>
