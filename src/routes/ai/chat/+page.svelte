<script lang="ts">
  import { onMount } from 'svelte';
  import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';
  import ConversationList from '$lib/domains/ai/components/conversations/ConversationList.svelte';
  import ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
  import PlatformOfflineBanner from '$lib/domains/ai/components/PlatformOfflineBanner.svelte';
  import PlatformStatusBanner from '$lib/domains/ai/components/PlatformStatusBanner.svelte';
  import AIContextBar from '$lib/domains/ai/components/AIContextBar.svelte';
  import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
  import { AIChatPage } from '$lib/domains/ai/state/aiChatPage.svelte.js';
  import PanelLeftOpen from '~icons/fa6-solid/table-columns';
  import Settings2 from '~icons/fa6-solid/sliders';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import { Button } from '$lib/components/ui/button';
  import ChatSettingsPanel from '$lib/domains/ai/components/chat/ChatSettingsPanel.svelte';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';

  let providerSelector = $state<ProviderModelSelector | null>(null);
  const chat = new AIChatPage(() => providerSelector);

  onMount(chat.init);

  // Same deal as the coder page: park the controls in the AI layout's tab row.
  $effect(() => {
    aiTopbar.actions = topbarActions;
    return () => {
      aiTopbar.actions = null;
    };
  });
</script>

{#snippet topbarActions()}
  <AIContextBar contextUsage={chat.contextUsage} llmUsage={chat.llmUsage} variant="ring" />
  <ProviderModelSelector
    bind:this={providerSelector}
    bind:selectedProvider={chat.selectedProvider}
    bind:selectedBackendProvider={chat.selectedBackendProvider}
    bind:selectedModel={chat.selectedModel}
    onModelChange={chat.handleModelChange}
    onStatusChange={chat.applyPlatformStatus}
    showInlineError={false}
    backendSelectClass="w-[130px]"
    modelSelectClass="w-[220px]"
  />
  <Button
    size="icon"
    variant={chat.settingsPanelOpen ? 'secondary' : 'ghost'}
    class="h-8 w-8 shrink-0"
    title="Chat settings"
    onclick={() => (chat.settingsPanelOpen = !chat.settingsPanelOpen)}
  >
    <Settings2 class="h-4 w-4" />
  </Button>
{/snippet}

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel bind:open={chat.conversationsPanelOpen} side="left" desktopClass="w-64">
    <ConversationList
      bind:conversations={chat.conversations}
      onConversationClick={(c) => {
        chat.handleConversationClick(c);
        chat.conversationsPanelOpen = false;
      }}
      onCreateNew={chat.handleNewConversation}
      onDeleteConversation={chat.handleDeleteConversation}
      onDeleteAll={chat.handleDeleteAllConversations}
      selectedConversationId={chat.selectedConversation?.id}
    />
  </ResponsivePanel>

  <!-- Main Chat Area -->
  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <div
      class="divider-edge-b divider-edge-full flex flex-wrap items-center justify-between gap-2 bg-background px-4 py-2.5"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8 shrink-0 md:hidden"
          title="Conversations"
          onclick={() => (chat.conversationsPanelOpen = true)}
        >
          <PanelLeftOpen class="h-4 w-4" />
        </Button>
        <h2 class="truncate text-sm font-semibold">
          {chat.selectedConversation?.title || 'AI Chat'}
        </h2>
      </div>
    </div>
    {#if aiPlatform.blocked}
      <PlatformStatusBanner blocks="Sending, regenerate and edit-and-resend" />
    {:else if chat.platformOffline}
      <PlatformOfflineBanner
        unauthorized={chat.platformUnauthorized}
        checking={chat.platformChecking}
        error={chat.platformError}
        onRetry={chat.retryPlatform}
      />
    {/if}
    <div class="flex min-h-0 w-full flex-1 flex-col">
      <AIChatPanel
        bind:messages={chat.messages}
        isLoading={chat.isSending || chat.isLoading}
        title={chat.selectedConversation?.title || 'AI Chat'}
        placeholder="Ask me anything..."
        modelLabel={chat.selectedModel}
        conversationId={chat.conversationId}
        showSelectors={false}
        contextUsage={chat.contextUsage}
        settingsOpen={chat.settingsPanelOpen}
        onOpenSettings={() => (chat.settingsPanelOpen = !chat.settingsPanelOpen)}
        onSendMessageWithHistory={chat.handleSendMessage}
        onStop={chat.handleStop}
        onRegenerate={chat.handleRegenerate}
        onTruncate={chat.handleTruncate}
        onBranch={chat.handleBranch}
        sendDisabled={chat.platformOffline}
        sendDisabledHint={chat.platformUnauthorized
          ? 'Sending is disabled: the agent-platform token was rejected. Fix it in Settings > AI.'
          : 'Agent Platform is offline — retry to reconnect.'}
      />
    </div>
  </div>

  {#if chat.settingsPanelOpen}
    <ResponsivePanel bind:open={chat.settingsPanelOpen} side="right" desktopClass="w-72">
      {#snippet header()}
        <div class="divider-edge-b divider-edge-full p-2.5">
          <h2 class="flex items-center gap-1.5 text-sm font-semibold">
            <Settings2 class="h-4 w-4" />
            Model Parameters
          </h2>
        </div>
      {/snippet}
      <ChatSettingsPanel bind:settings={chat.chatSettings} />
    </ResponsivePanel>
  {/if}
</div>
