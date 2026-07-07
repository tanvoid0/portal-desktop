<!--
	Learning Settings - ML Learning System Configuration
-->

<script lang="ts">
  import { onMount } from "svelte";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import { Switch } from "$lib/components/ui/switch";
  import { Separator } from "$lib/components/ui/separator";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import Select from "$lib/components/ui/select.svelte";
  import {
    learningService,
    memoryService,
    type MLIntensity,
    type MemoryStats,
    type CleanupPreview,
    type LearnedPattern,
    type LearningEvent,
    type UserPreference,
  } from "$lib/domains/learning";
  import {
    Tabs,
    TabsList,
    TabsTrigger,
    TabsContent,
  } from "$lib/components/ui/tabs";
  import { logger } from "$lib/domains/shared/services/logger";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import { goto } from "$app/navigation";
  import {
    Brain,
    Activity,
    Database,
    Trash2,
    Sparkles,
    Zap,
    Settings,
    CheckCircle2,
    XCircle,
    RefreshCw,
    Eye,
    Star,
    TrendingUp,
    Clock,
    Filter,
    Search,
  } from "@lucide/svelte";

  const log = logger.createScoped("LearningSettings");

  let mlEnabled = $state(true);
  let mlIntensity = $state<MLIntensity | null>(null); // Start as null, will be set after load
  let memoryStats = $state<MemoryStats | null>(null);
  let cleanupPreview = $state<CleanupPreview | null>(null);
  let isLoading = $state(false);
  let isRefreshing = $state(false);
  let lastUpdate = $state<Date | null>(null);

  // Dashboard data
  let patterns = $state<LearnedPattern[]>([]);
  let events = $state<LearningEvent[]>([]);
  let preferences = $state<UserPreference[]>([]);
  let isLoadingDashboard = $state(false);
  let activeTab = $state<"patterns" | "events" | "preferences">("patterns");
  let patternsFilter = $state<
    "all" | "command" | "framework" | "workflow" | "code"
  >("all");
  let searchQuery = $state("");

  const intensityOptions = [
    {
      value: "fast" as MLIntensity,
      label: "Fast",
      description: "Minimal learning, basic patterns only",
      iconName: "zap",
    },
    {
      value: "light" as MLIntensity,
      label: "Light",
      description: "Limited analysis, basic suggestions",
      iconName: "sparkles",
    },
    {
      value: "medium" as MLIntensity,
      label: "Medium",
      description: "Full analysis, contextual suggestions",
      iconName: "brain",
    },
    {
      value: "deep" as MLIntensity,
      label: "Deep",
      description: "Advanced analysis, predictive actions",
      iconName: "activity",
    },
  ];

  function getIntensityIcon(intensity: MLIntensity) {
    switch (intensity) {
      case "fast":
        return Zap;
      case "light":
        return Sparkles;
      case "medium":
        return Brain;
      case "deep":
        return Activity;
      default:
        return Sparkles;
    }
  }

  const intensityDescriptions: Record<MLIntensity, string> = {
    fast: "Processes in large batches every 60 seconds. Minimal resource usage.",
    light: "Processes in medium batches every 30 seconds. Low resource usage.",
    medium:
      "Processes in small batches every 15 seconds. Moderate resource usage.",
    deep: "Processes in very small batches every 5 seconds. Higher resource usage.",
  };

  onMount(async () => {
    await loadSettings();
    await loadDashboardData();
  });

  async function loadSettings() {
    isLoading = true;
    try {
      [mlEnabled, mlIntensity, memoryStats, cleanupPreview] = await Promise.all(
        [
          learningService.getMLEnabled(),
          learningService.getMLIntensity(),
          memoryService.getStats().catch(() => null),
          memoryService.getCleanupPreview().catch(() => null),
        ],
      );
      lastUpdate = new Date();
      log.info("Learning settings loaded");
    } catch (error) {
      log.error("Failed to load learning settings", error);
      toast.error("Failed to load learning settings");
    } finally {
      isLoading = false;
    }
  }

  async function loadDashboardData() {
    isLoadingDashboard = true;
    try {
      [patterns, events, preferences] = await Promise.all([
        learningService.getAllPatterns(100).catch(() => []),
        learningService.getRecentEvents(50).catch(() => []),
        learningService.getAllPreferences().catch(() => []),
      ]);
      log.info("Dashboard data loaded", {
        patterns: patterns.length,
        events: events.length,
        preferences: preferences.length,
      });
    } catch (error) {
      log.error("Failed to load dashboard data", error);
      toast.error("Failed to load learning data");
    } finally {
      isLoadingDashboard = false;
    }
  }

  async function handleToggleEnabled() {
    try {
      await learningService.setMLEnabled(mlEnabled);
      toast.success(`ML Learning ${mlEnabled ? "enabled" : "disabled"}`);
    } catch (error) {
      log.error("Failed to toggle ML enabled state", error);
      toast.error("Failed to update ML enabled state");
      mlEnabled = !mlEnabled; // Revert
    }
  }

  async function handleIntensityChange(newIntensity: MLIntensity) {
    try {
      mlIntensity = newIntensity;
      await learningService.setMLIntensity(newIntensity);
      toast.success(
        `ML Intensity set to ${intensityOptions.find((o) => o.value === newIntensity)?.label}`,
      );
    } catch (error) {
      log.error("Failed to set ML intensity", error);
      toast.error("Failed to update ML intensity");
      await loadSettings(); // Reload to revert
    }
  }

  async function handleCleanup() {
    if (!cleanupPreview) return;

    const totalToDelete =
      cleanupPreview.events_to_delete + cleanupPreview.patterns_to_delete;
    if (totalToDelete === 0) {
      toast.info("No data to clean up");
      return;
    }

    const confirmed = await confirmAction(
      `This will delete ${totalToDelete} items. Continue?`,
      "Clean up memory",
    );
    if (!confirmed) return;

    isRefreshing = true;
    try {
      await memoryService.cleanup();
      toast.success("Memory cleanup completed");
      await refreshStats();
    } catch (error) {
      log.error("Failed to cleanup memory", error);
      toast.error("Failed to cleanup memory");
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
      log.error("Failed to refresh stats", error);
    } finally {
      isRefreshing = false;
    }
  }

  function formatNumber(num: number): string {
    return new Intl.NumberFormat().format(num);
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return "Never";
    try {
      const date = new Date(dateStr);
      return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }).format(date);
    } catch {
      return dateStr;
    }
  }

  function formatPatternData(data: Record<string, unknown>): string {
    try {
      return JSON.stringify(data, null, 2);
    } catch {
      return String(data);
    }
  }

  function getPatternTypeLabel(type: string): string {
    return type.charAt(0).toUpperCase() + type.slice(1);
  }

  function getEventTypeLabel(type: string): string {
    return type
      .split("_")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join(" ");
  }

  async function togglePatternImportant(
    patternId: number,
    isImportant: boolean,
  ) {
    try {
      await learningService.markPatternImportant(patternId, isImportant);
      // Reload patterns
      patterns = await learningService.getAllPatterns(100).catch(() => []);
      toast.success(
        `Pattern marked as ${isImportant ? "important" : "not important"}`,
      );
    } catch (error) {
      log.error("Failed to toggle pattern importance", error);
      toast.error("Failed to update pattern");
    }
  }

  const filteredPatterns = $derived.by(() => {
    let filtered = patterns;

    if (patternsFilter !== "all") {
      filtered = filtered.filter((p) => p.pattern_type === patternsFilter);
    }

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((p) => {
        const dataStr = JSON.stringify(p.pattern_data).toLowerCase();
        const contextStr = (p.context || "").toLowerCase();
        return dataStr.includes(query) || contextStr.includes(query);
      });
    }

    return filtered;
  });
</script>

<div class="space-y-6">
  <!-- AI Provider Settings Redirect -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Sparkles class="h-5 w-5" />
        AI Provider Configuration
      </CardTitle>
      <CardDescription>
        AI provider settings have been moved to the dedicated AI page
      </CardDescription>
    </CardHeader>
    <CardContent>
      <Button onclick={() => goto("/settings/ai")}>
        <Sparkles class="mr-2 h-4 w-4" />
        Go to AI Providers
      </Button>
    </CardContent>
  </Card>

  <Separator />

  <!-- Status Card -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Activity class="h-5 w-5" />
        Learning System Status
      </CardTitle>
      <CardDescription>
        Current state and activity of the ML learning system
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Label for="ml-enabled" class="text-base font-medium"
            >ML Learning Enabled</Label
          >
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
            <div class="mt-1 h-6 animate-pulse rounded bg-muted"></div>
          {:else if mlIntensity === "fast"}
            <div class="mt-1 flex items-center gap-2">
              <Zap class="h-4 w-4" />
              <span class="font-medium">Fast</span>
            </div>
          {:else if mlIntensity === "light"}
            <div class="mt-1 flex items-center gap-2">
              <Sparkles class="h-4 w-4" />
              <span class="font-medium">Light</span>
            </div>
          {:else if mlIntensity === "medium"}
            <div class="mt-1 flex items-center gap-2">
              <Brain class="h-4 w-4" />
              <span class="font-medium">Medium</span>
            </div>
          {:else if mlIntensity === "deep"}
            <div class="mt-1 flex items-center gap-2">
              <Activity class="h-4 w-4" />
              <span class="font-medium">Deep</span>
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
    </CardContent>
  </Card>

  <!-- Intensity Control -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Settings class="h-5 w-5" />
        Resource Usage Control
      </CardTitle>
      <CardDescription>
        Balance between learning intelligence and system performance
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <Label>ML Intensity Level</Label>
        {#if mlIntensity !== null}
          <Select
            options={intensityOptions.map((o) => ({
              value: o.value,
              label: `${o.label} - ${o.description}`,
            }))}
            defaultValue={mlIntensity}
            onSelect={(value) =>
              value && handleIntensityChange(value as MLIntensity)}
            disabled={!mlEnabled || isLoading}
          />
        {:else}
          <div
            class="h-10 animate-pulse rounded-md border border-input bg-muted"
          ></div>
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
    </CardContent>
  </Card>

  <!-- Memory Statistics -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Database class="h-5 w-5" />
        Memory Usage
      </CardTitle>
      <CardDescription>Storage and retention statistics</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if memoryStats}
        <div class="grid grid-cols-3 gap-4">
          <div>
            <Label class="text-sm text-muted-foreground">Learning Events</Label>
            <div class="mt-1 text-2xl font-bold">
              {formatNumber(memoryStats.total_events)}
            </div>
            <div class="text-xs text-muted-foreground">
              Retention: {memoryStats.events_retention_days} days
            </div>
          </div>
          <div>
            <Label class="text-sm text-muted-foreground">Patterns Learned</Label
            >
            <div class="mt-1 text-2xl font-bold">
              {formatNumber(memoryStats.total_patterns)}
            </div>
            <div class="text-xs text-muted-foreground">
              Max: {formatNumber(memoryStats.max_patterns_per_type)} per type
            </div>
          </div>
          <div>
            <Label class="text-sm text-muted-foreground">Preferences</Label>
            <div class="mt-1 text-2xl font-bold">
              {formatNumber(memoryStats.total_preferences)}
            </div>
            <div class="text-xs text-muted-foreground">
              User preferences stored
            </div>
          </div>
        </div>

        {#if cleanupPreview}
          <Separator />
          <div class="space-y-2">
            <Label class="text-sm">Pending Cleanup</Label>
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">
                {cleanupPreview.events_to_delete} events, {cleanupPreview.patterns_to_delete}
                patterns
              </span>
              <Button
                variant="outline"
                size="sm"
                onclick={handleCleanup}
                disabled={isRefreshing ||
                  (cleanupPreview.events_to_delete === 0 &&
                    cleanupPreview.patterns_to_delete === 0)}
              >
                <Trash2 class="mr-2 h-4 w-4" />
                Clean Up
              </Button>
            </div>
          </div>
        {/if}

        <Separator />

        <div class="flex justify-end">
          <Button
            variant="ghost"
            size="sm"
            onclick={refreshStats}
            disabled={isRefreshing}
          >
            <RefreshCw
              class="mr-2 h-4 w-4 {isRefreshing ? 'animate-spin' : ''}"
            />
            Refresh Stats
          </Button>
        </div>
      {:else}
        <div class="py-8 text-center text-muted-foreground">
          {#if isLoading}
            Loading statistics...
          {:else}
            Failed to load statistics
          {/if}
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Learning Dashboard -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Eye class="h-5 w-5" />
        Learning Dashboard
      </CardTitle>
      <CardDescription>
        View learned patterns, events, and preferences
      </CardDescription>
    </CardHeader>
    <CardContent>
      <Tabs bind:value={activeTab}>
        <TabsList class="grid w-full grid-cols-3">
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

        <!-- Patterns Tab -->
        <TabsContent value="patterns" class="mt-4 space-y-4">
          {#if isLoadingDashboard}
            <div class="py-8 text-center text-muted-foreground">
              Loading patterns...
            </div>
          {:else if patterns.length === 0}
            <div class="py-8 text-center text-muted-foreground">
              <p>No patterns learned yet.</p>
              <p class="mt-2 text-xs">
                Patterns will appear here as you use the application.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              <div class="flex items-center gap-2">
                <div class="flex flex-1 items-center gap-2">
                  <Search class="h-4 w-4 text-muted-foreground" />
                  <Input
                    bind:value={searchQuery}
                    placeholder="Search patterns..."
                    class="max-w-sm flex-1"
                  />
                </div>
                <Select
                  options={[
                    { value: "all", label: "All Types" },
                    { value: "command", label: "Commands" },
                    { value: "framework", label: "Frameworks" },
                    { value: "workflow", label: "Workflows" },
                    { value: "code", label: "Code" },
                  ]}
                  defaultValue={patternsFilter}
                  onSelect={(value) =>
                    (patternsFilter = (value ||
                      "all") as typeof patternsFilter)}
                />
              </div>

              <div class="max-h-[600px] space-y-2 overflow-y-auto">
                {#each filteredPatterns as pattern (pattern.id)}
                  <Card class="p-4">
                    <div class="flex items-start justify-between gap-4">
                      <div class="flex-1 space-y-2">
                        <div class="flex items-center gap-2">
                          <Badge variant="outline">
                            {getPatternTypeLabel(pattern.pattern_type)}
                          </Badge>
                          {#if pattern.is_important}
                            <Badge variant="default" class="bg-yellow-500">
                              <Star class="mr-1 h-3 w-3" />
                              Important
                            </Badge>
                          {/if}
                          {#if pattern.context}
                            <Badge variant="secondary">
                              {pattern.context}
                            </Badge>
                          {/if}
                        </div>
                        <div
                          class="max-h-32 overflow-y-auto rounded bg-muted p-2 font-mono text-sm text-xs"
                        >
                          {formatPatternData(pattern.pattern_data)}
                        </div>
                        <div
                          class="flex items-center gap-4 text-xs text-muted-foreground"
                        >
                          <div class="flex items-center gap-1">
                            <TrendingUp class="h-3 w-3" />
                            Used {pattern.frequency} times
                          </div>
                          <div class="flex items-center gap-1">
                            {#if pattern.success_rate >= 0.7}
                              <CheckCircle2 class="h-3 w-3 text-green-500" />
                            {:else}
                              <XCircle class="h-3 w-3 text-red-500" />
                            {/if}
                            {(pattern.success_rate * 100).toFixed(0)}% success
                          </div>
                          {#if pattern.last_used}
                            <div class="flex items-center gap-1">
                              <Clock class="h-3 w-3" />
                              {formatDate(pattern.last_used)}
                            </div>
                          {/if}
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() =>
                          togglePatternImportant(
                            pattern.id,
                            !pattern.is_important,
                          )}
                        title={pattern.is_important
                          ? "Mark as not important"
                          : "Mark as important"}
                      >
                        <Star
                          class="h-4 w-4"
                          fill={pattern.is_important ? "currentColor" : "none"}
                        />
                      </Button>
                    </div>
                  </Card>
                {/each}
              </div>

              {#if filteredPatterns.length === 0 && patterns.length > 0}
                <div class="py-4 text-center text-sm text-muted-foreground">
                  No patterns match your filter
                </div>
              {/if}
            </div>
          {/if}
        </TabsContent>

        <!-- Events Tab -->
        <TabsContent value="events" class="mt-4 space-y-4">
          {#if isLoadingDashboard}
            <div class="py-8 text-center text-muted-foreground">
              Loading events...
            </div>
          {:else if events.length === 0}
            <div class="py-8 text-center text-muted-foreground">
              <p>No events recorded yet.</p>
              <p class="mt-2 text-xs">
                Events will appear here as the system learns from your actions.
              </p>
            </div>
          {:else}
            <div class="max-h-[600px] space-y-2 overflow-y-auto">
              {#each events as event}
                <Card class="p-3">
                  <div class="flex items-start justify-between gap-4">
                    <div class="flex-1 space-y-1">
                      <div class="flex items-center gap-2">
                        <Badge variant="outline">
                          {getEventTypeLabel(event.event_type)}
                        </Badge>
                        {#if event.outcome}
                          <Badge
                            variant={event.outcome === "success"
                              ? "default"
                              : "destructive"}
                          >
                            {event.outcome}
                          </Badge>
                        {/if}
                        {#if event.context}
                          <Badge variant="secondary">
                            {event.context}
                          </Badge>
                        {/if}
                      </div>
                      <div
                        class="max-h-24 overflow-y-auto rounded bg-muted p-2 font-mono text-xs"
                      >
                        {formatPatternData(event.event_data)}
                      </div>
                      <div
                        class="flex items-center gap-1 text-xs text-muted-foreground"
                      >
                        <Clock class="h-3 w-3" />
                        {formatDate(event.created_at)}
                      </div>
                    </div>
                  </div>
                </Card>
              {/each}
            </div>
          {/if}
        </TabsContent>

        <!-- Preferences Tab -->
        <TabsContent value="preferences" class="mt-4 space-y-4">
          {#if isLoadingDashboard}
            <div class="py-8 text-center text-muted-foreground">
              Loading preferences...
            </div>
          {:else if preferences.length === 0}
            <div class="py-8 text-center text-muted-foreground">
              <p>No preferences learned yet.</p>
              <p class="mt-2 text-xs">
                Preferences will appear here as the system learns your
                preferences.
              </p>
            </div>
          {:else}
            <div class="max-h-[600px] space-y-2 overflow-y-auto">
              {#each preferences as pref}
                <Card class="p-3">
                  <div class="flex items-start justify-between gap-4">
                    <div class="flex-1 space-y-1">
                      <div class="flex items-center gap-2">
                        <Badge variant="outline">
                          {pref.preference_type}
                        </Badge>
                        {#if pref.is_important}
                          <Badge variant="default" class="bg-yellow-500">
                            <Star class="mr-1 h-3 w-3" />
                            Important
                          </Badge>
                        {/if}
                        {#if pref.context}
                          <Badge variant="secondary">
                            {pref.context}
                          </Badge>
                        {/if}
                        <Badge variant="outline">
                          {(pref.confidence * 100).toFixed(0)}% confidence
                        </Badge>
                      </div>
                      <div class="rounded bg-muted p-2 font-mono text-sm">
                        {formatPatternData(pref.preference_value)}
                      </div>
                      <div
                        class="flex items-center gap-4 text-xs text-muted-foreground"
                      >
                        {#if pref.learned_from}
                          <span>Learned from: {pref.learned_from}</span>
                        {/if}
                        <span>Updated: {formatDate(pref.updated_at)}</span>
                      </div>
                    </div>
                  </div>
                </Card>
              {/each}
            </div>
          {/if}
        </TabsContent>
      </Tabs>

      <Separator class="my-4" />

      <div class="flex justify-end">
        <Button
          variant="ghost"
          size="sm"
          onclick={loadDashboardData}
          disabled={isLoadingDashboard}
        >
          <RefreshCw
            class="mr-2 h-4 w-4 {isLoadingDashboard ? 'animate-spin' : ''}"
          />
          Refresh Dashboard
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
