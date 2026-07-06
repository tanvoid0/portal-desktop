<!--
	AI Hub - Main dashboard for AI features
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    MessageSquare,
    Server,
    CheckCircle2,
    ArrowRight,
    ChevronRight,
    Settings,
    Database,
    History,
  } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { ProviderType, Conversation } from "$lib/domains/ai/types";
  import { PageHeader, PageLoading, PageError } from "$lib/components/shell";

  let defaultProvider = $state<ProviderType | null>(null);
  let recentConversations = $state<Conversation[]>([]);
  let isLoading = $state(true);
  let loadError = $state<string | null>(null);

  const quickActions = [
    {
      title: "Start Chat",
      description: "Begin a new conversation with AI",
      icon: MessageSquare,
      url: "/ai/chat",
      color: "text-primary",
    },
    {
      title: "Configure Providers",
      description: "Set up Ollama, OpenAI, and other AI providers",
      icon: Settings,
      url: "/ai/providers",
      color: "text-primary",
    },
    {
      title: "Training Data",
      description: "Manage AI training datasets",
      icon: Database,
      url: "/ai/training",
      color: "text-primary",
    },
    {
      title: "Conversation History",
      description: "Browse past AI conversations",
      icon: History,
      url: "/ai/history",
      color: "text-primary",
    },
  ];

  async function loadHubData() {
    isLoading = true;
    loadError = null;

    try {
      const provider = await invoke<ProviderType | null>(
        "get_default_ai_provider",
      );
      defaultProvider = provider;

      const conversations = await invoke<Conversation[]>(
        "ai_list_conversations",
        { limit: 5 },
      );
      recentConversations = conversations || [];
    } catch (error) {
      loadError =
        error instanceof Error ? error.message : "Failed to load AI hub data";
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    void loadHubData();
  });
</script>

<svelte:head>
  <title>AI Hub - Portal Desktop</title>
</svelte:head>

<div class="h-full w-full overflow-y-auto p-6">
  <div class="mx-auto max-w-7xl space-y-6">
    <PageHeader
      title="AI Hub"
      description="Manage AI providers, conversations, and training data"
    >
      {#snippet actions()}
        <Button onclick={() => goto("/ai/chat")}>
          <MessageSquare class="mr-2 h-4 w-4" />
          New Chat
        </Button>
      {/snippet}
    </PageHeader>

    {#if isLoading}
      <PageLoading message="Loading AI hub..." />
    {:else if loadError}
      <PageError
        title="Unable to load AI hub"
        message={loadError}
        onRetry={loadHubData}
      />
    {:else}
      <!-- Quick Stats -->
      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Default Provider</CardTitle>
            <Server class="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">
              {defaultProvider || "Not Set"}
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {#if defaultProvider}
                Configured and ready
              {:else}
                <a href="/ai/providers" class="text-primary hover:underline"
                  >Set up a provider</a
                >
              {/if}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium"
              >Recent Conversations</CardTitle
            >
            <MessageSquare class="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">{recentConversations.length}</div>
            <p class="mt-1 text-xs text-muted-foreground">
              <a href="/ai/chat" class="text-primary hover:underline"
                >View all</a
              >
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <CardTitle class="text-sm font-medium">Status</CardTitle>
            <CheckCircle2 class="h-4 w-4 text-status-success" />
          </CardHeader>
          <CardContent>
            <div class="text-2xl font-bold">
              <Badge variant="default">Ready</Badge>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              AI services operational
            </p>
          </CardContent>
        </Card>
      </div>

      <!-- Quick Actions -->
      <div>
        <h2 class="mb-4 text-xl font-semibold">Quick Actions</h2>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each quickActions as action}
            <Card
              class="cursor-pointer transition-shadow hover:shadow-md"
              onclick={() => goto(action.url)}
            >
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  {@const IconComponent = action.icon}
                  <IconComponent class="h-5 w-5 {action.color}" />
                  {action.title}
                </CardTitle>
                <CardDescription>{action.description}</CardDescription>
              </CardHeader>
            </Card>
          {/each}
        </div>
      </div>

      <!-- Recent Conversations -->
      {#if recentConversations.length > 0}
        <div>
          <div class="mb-4 flex items-center justify-between">
            <h2 class="text-xl font-semibold">Recent Conversations</h2>
            <Button variant="ghost" size="sm" onclick={() => goto("/ai/chat")}>
              View All
              <ArrowRight class="ml-1 h-4 w-4" />
            </Button>
          </div>
          <div class="space-y-2">
            {#each recentConversations as conversation}
              <Card
                class="cursor-pointer transition-shadow hover:shadow-md"
                onclick={() => goto(`/ai/chat?id=${conversation.id}`)}
              >
                <CardContent class="p-4">
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <h3 class="font-medium">{conversation.title}</h3>
                      <div class="mt-1 flex items-center gap-2">
                        <Badge variant="outline">{conversation.provider}</Badge>
                        <span class="text-xs text-muted-foreground">
                          {new Date(
                            conversation.updated_at,
                          ).toLocaleDateString()}
                        </span>
                        {#if conversation.message_count}
                          <span class="text-xs text-muted-foreground">
                            {conversation.message_count} messages
                          </span>
                        {/if}
                      </div>
                    </div>
                    <ChevronRight class="h-5 w-5 text-muted-foreground" />
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>
