<!--
	Autonomy Settings - Autonomous Action System Configuration
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Separator } from '$lib/components/ui/separator';
  import { Badge } from '$lib/components/ui/badge';
  import Select from '$lib/components/ui/select.svelte';
  import { autonomyService, type AutonomyLevel, type ApprovalStats } from '$lib/domains/autonomy';
  import { logger } from '$lib/domains/shared/services/logger';
  import { toast } from '$lib/utils/toast';
  import { PageError } from '$lib/components/shell';
  import { levelOptions, levelDescriptions } from './autonomy/levels';
  import SettingsSection from './shared/SettingsSection.svelte';
  import AutonomyApprovalStats from './autonomy/AutonomyApprovalStats.svelte';
  import AutonomyInfoCard from './autonomy/AutonomyInfoCard.svelte';
  import Bot from '~icons/fa6-solid/robot';
  import CheckCircle2 from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import AlertCircle from '~icons/fa6-solid/circle-exclamation';
  import Settings from '~icons/fa6-solid/gear';

  const log = logger.createScoped('AutonomySettings');

  let autonomyEnabled = $state(false);
  let autonomyLevel = $state<AutonomyLevel | null>(null);
  let approvalStats = $state<ApprovalStats | null>(null);
  let loadError = $state<string | null>(null);
  let statsError = $state<string | null>(null);
  let isLoading = $state(false);
  let isRefreshing = $state(false);

  onMount(async () => {
    await load();
  });

  async function load() {
    await loadSettings();
    if (!loadError) await loadStats();
  }

  async function loadSettings() {
    isLoading = true;
    try {
      [autonomyEnabled, autonomyLevel] = await Promise.all([
        autonomyService.getEnabled(),
        autonomyService.getLevel(),
      ]);
      loadError = null;
      log.info('Autonomy settings loaded', { autonomyEnabled, autonomyLevel });
    } catch (error) {
      log.error('Failed to load autonomy settings', error);
      loadError = error instanceof Error ? error.message : String(error);
    } finally {
      isLoading = false;
    }
  }

  async function loadStats() {
    try {
      approvalStats = await autonomyService.getApprovalStats();
      statsError = null;
      log.info('Approval stats loaded');
    } catch (error) {
      log.error('Failed to load approval stats', error);
      statsError = error instanceof Error ? error.message : String(error);
    }
  }

  async function handleToggleEnabled(enabled: boolean) {
    autonomyEnabled = enabled;
    try {
      await autonomyService.setEnabled(autonomyEnabled);
      toast.success(`Autonomy ${autonomyEnabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      log.error('Failed to toggle autonomy enabled state', error);
      toast.error('Failed to update autonomy enabled state');
      autonomyEnabled = !autonomyEnabled; // Revert
    }
  }

  async function handleLevelChange(newLevel: AutonomyLevel) {
    try {
      autonomyLevel = newLevel;
      await autonomyService.setLevel(newLevel);
      toast.success(
        `Autonomy level set to ${levelOptions.find((o) => o.value === newLevel)?.label}`
      );
    } catch (error) {
      log.error('Failed to set autonomy level', error);
      toast.error('Failed to update autonomy level');
      await loadSettings(); // Reload to revert
    }
  }

  async function refreshStats() {
    isRefreshing = true;
    try {
      await loadStats();
    } catch (error) {
      log.error('Failed to refresh stats', error);
    } finally {
      isRefreshing = false;
    }
  }
</script>

{#if loadError}
  <PageError title="Autonomy settings unavailable" message={loadError} onRetry={load} />
{:else}
  <div class="space-y-6">
    <SettingsSection
      title="Autonomous Action System"
      description="Control how the system autonomously executes actions based on learned patterns"
      icon={Bot}
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Label for="autonomy-enabled" class="text-base font-medium">Autonomy Enabled</Label>
          {#if autonomyEnabled}
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
          id="autonomy-enabled"
          checked={autonomyEnabled}
          onCheckedChange={handleToggleEnabled}
          disabled={isLoading}
        />
      </div>

      <Separator />

      <div class="space-y-2">
        <Label>Current Autonomy Level</Label>
        {#if autonomyLevel === null}
          <div class="h-10 animate-pulse rounded-md border border-input bg-muted"></div>
        {:else}
          {#each levelOptions as option}
            {#if option.value === autonomyLevel}
              {@const Icon = option.icon}
              <div class="flex items-center gap-2 rounded-md border bg-muted/50 p-3">
                <Icon class="h-5 w-5 {option.color}" />
                <div class="flex-1">
                  <div class="font-medium">{option.label}</div>
                  <div class="text-sm text-muted-foreground">
                    {option.description}
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    </SettingsSection>

    <SettingsSection
      title="Autonomy Level"
      description="Choose how aggressively the system executes autonomous actions"
      icon={Settings}
    >
      <div class="space-y-2">
        <Label>Select Autonomy Level</Label>
        {#if autonomyLevel !== null}
          <Select
            options={levelOptions.map((o) => ({
              value: o.value,
              label: `${o.label} - ${o.description}`,
            }))}
            defaultValue={autonomyLevel}
            onSelect={(value) => value && handleLevelChange(value as AutonomyLevel)}
            disabled={!autonomyEnabled || isLoading}
          />
        {:else}
          <div class="h-10 animate-pulse rounded-md border border-input bg-muted"></div>
        {/if}
        {#if autonomyLevel !== null}
          <div class="rounded-md bg-muted p-3 text-sm text-muted-foreground">
            {levelDescriptions[autonomyLevel]}
          </div>
        {/if}
      </div>

      {#if !autonomyEnabled}
        <div class="rounded-md bg-muted p-3 text-sm text-muted-foreground">
          Enable autonomy above to adjust level settings.
        </div>
      {/if}

      {#if autonomyLevel === 'aggressive'}
        <div class="rounded-md border border-orange-500/20 bg-orange-500/10 p-3">
          <div class="flex items-start gap-2">
            <AlertCircle class="mt-0.5 h-4 w-4 flex-shrink-0 text-orange-500" />
            <div class="text-sm">
              <strong class="text-orange-700 dark:text-orange-400">Warning:</strong>
              <span class="text-orange-600 dark:text-orange-300">
                {' '}Aggressive mode allows medium-risk actions to execute automatically. Monitor
                the system closely and review approval statistics regularly.
              </span>
            </div>
          </div>
        </div>
      {/if}
    </SettingsSection>

    <AutonomyApprovalStats
      stats={approvalStats}
      error={statsError}
      {isRefreshing}
      onRefresh={refreshStats}
    />

    <AutonomyInfoCard />
  </div>
{/if}
