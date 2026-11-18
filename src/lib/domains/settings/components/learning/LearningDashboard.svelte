<!-- Learning dashboard: tabs for learned patterns, events and preferences. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Separator } from '$lib/components/ui/separator';
  import { Button } from '$lib/components/ui/button';
  import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
  import {
    learningService,
    type LearnedPattern,
    type LearningEvent,
    type UserPreference,
  } from '$lib/domains/learning';
  import { logger } from '$lib/domains/shared/services/logger';
  import { toast } from '$lib/utils/toast';
  import Activity from '~icons/fa6-solid/wave-square';
  import Eye from '~icons/fa6-solid/eye';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import Star from '~icons/fa6-solid/star';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import SettingsSection from '../shared/SettingsSection.svelte';
  import LearningPatternsTab from './LearningPatternsTab.svelte';
  import LearningEventsTab from './LearningEventsTab.svelte';
  import LearningPreferencesTab from './LearningPreferencesTab.svelte';

  const log = logger.createScoped('LearningDashboard');

  let patterns = $state<LearnedPattern[]>([]);
  let events = $state<LearningEvent[]>([]);
  let preferences = $state<UserPreference[]>([]);
  let isLoadingDashboard = $state(false);
  let activeTab = $state<'patterns' | 'events' | 'preferences'>('patterns');

  onMount(loadDashboardData);

  async function loadDashboardData() {
    isLoadingDashboard = true;
    try {
      [patterns, events, preferences] = await Promise.all([
        learningService.getAllPatterns(100).catch(() => []),
        learningService.getRecentEvents(50).catch(() => []),
        learningService.getAllPreferences().catch(() => []),
      ]);
      log.info('Dashboard data loaded', {
        patterns: patterns.length,
        events: events.length,
        preferences: preferences.length,
      });
    } catch (error) {
      log.error('Failed to load dashboard data', error);
      toast.error('Failed to load learning data');
    } finally {
      isLoadingDashboard = false;
    }
  }

  async function togglePatternImportant(patternId: number, isImportant: boolean) {
    try {
      await learningService.markPatternImportant(patternId, isImportant);
      patterns = await learningService.getAllPatterns(100).catch(() => []);
      toast.success(`Pattern marked as ${isImportant ? 'important' : 'not important'}`);
    } catch (error) {
      log.error('Failed to toggle pattern importance', error);
      toast.error('Failed to update pattern');
    }
  }
</script>

{#snippet placeholder(loadingText: string, empty: string, emptyHint: string, count: number)}
  {#if isLoadingDashboard}
    <div class="py-8 text-center text-muted-foreground">{loadingText}</div>
  {:else if count === 0}
    <div class="py-8 text-center text-muted-foreground">
      <p>{empty}</p>
      <p class="mt-2 text-xs">{emptyHint}</p>
    </div>
  {/if}
{/snippet}

<SettingsSection
  title="Learning Dashboard"
  description="View learned patterns, events, and preferences"
  icon={Eye}
  contentClass=""
>
  <Tabs bind:value={activeTab}>
    <TabsList>
      <TabsTrigger value="patterns">
        <Sparkles class="mr-2 h-4 w-4" />
        Patterns ({patterns.length})
      </TabsTrigger>
      <TabsTrigger value="events">
        <Activity class="mr-2 h-4 w-4" />
        Events ({events.length})
      </TabsTrigger>
      <TabsTrigger value="preferences">
        <Star class="mr-2 h-4 w-4" />
        Preferences ({preferences.length})
      </TabsTrigger>
    </TabsList>

    <TabsContent value="patterns" class="mt-4 space-y-4">
      {@render placeholder(
        'Loading patterns...',
        'No patterns learned yet.',
        'Patterns will appear here as you use the application.',
        patterns.length
      )}
      {#if !isLoadingDashboard && patterns.length > 0}
        <LearningPatternsTab {patterns} onToggleImportant={togglePatternImportant} />
      {/if}
    </TabsContent>

    <TabsContent value="events" class="mt-4 space-y-4">
      {@render placeholder(
        'Loading events...',
        'No events recorded yet.',
        'Events will appear here as the system learns from your actions.',
        events.length
      )}
      {#if !isLoadingDashboard && events.length > 0}<LearningEventsTab {events} />{/if}
    </TabsContent>

    <TabsContent value="preferences" class="mt-4 space-y-4">
      {@render placeholder(
        'Loading preferences...',
        'No preferences learned yet.',
        'Preferences will appear here as the system learns your preferences.',
        preferences.length
      )}
      {#if !isLoadingDashboard && preferences.length > 0}
        <LearningPreferencesTab {preferences} />
      {/if}
    </TabsContent>
  </Tabs>

  <Separator class="my-4" />

  <div class="flex justify-end">
    <Button variant="ghost" size="sm" onclick={loadDashboardData} disabled={isLoadingDashboard}>
      <RefreshCw class="mr-2 h-4 w-4 {isLoadingDashboard ? 'animate-spin' : ''}" />
      Refresh Dashboard
    </Button>
  </div>
</SettingsSection>
