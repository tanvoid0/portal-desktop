<!--
	AI Hub - Main dashboard for AI features
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Sparkles, MessageSquare, Loader, Server, CheckCircle2, ArrowRight, ChevronRight, Settings, Database, FileText } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { ProviderType, Conversation } from '$lib/domains/ai/types';

	let defaultProvider = $state<ProviderType | null>(null);
	let recentConversations = $state<Conversation[]>([]);
	let isLoading = $state(true);

	const quickActions = [
		{
			title: 'Start Chat',
			description: 'Begin a new conversation with AI',
			icon: MessageSquare,
			url: '/ai/chat',
			color: 'text-blue-600'
		},
		{
			title: 'Configure Providers',
			description: 'Set up Ollama, OpenAI, and other AI providers',
			icon: Settings,
			url: '/ai/providers',
			color: 'text-purple-600'
		},
		{
			title: 'Training Data',
			description: 'Manage AI training datasets',
			icon: Database,
			url: '/ai/training',
			color: 'text-orange-600'
		},
		{
			title: 'View Logs',
			description: 'Monitor AI interactions and errors',
			icon: FileText,
			url: '/ai/logs',
			color: 'text-red-600'
		}
	];

	onMount(async () => {
		try {
			// Load default provider
			const provider = await invoke<ProviderType | null>('get_default_ai_provider');
			defaultProvider = provider;

			// Load recent conversations
			const conversations = await invoke<Conversation[]>('ai_list_conversations', {
				limit: 5
			});
			recentConversations = conversations || [];
		} catch (error) {
			// Error loading AI hub data - will show empty state
			// TODO: Add proper error logging when logger is available
		} finally {
			isLoading = false;
		}
	});
</script>

<div class="h-full w-full p-6 overflow-y-auto">
	<div class="max-w-7xl mx-auto space-y-6">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-3xl font-bold flex items-center gap-2">
					<Sparkles class="h-8 w-8" />
					AI Hub
				</h1>
				<p class="text-muted-foreground mt-1">
					Manage AI providers, conversations, and training data
				</p>
			</div>
			<Button onclick={() => goto('/ai/chat')}>
				<MessageSquare class="h-4 w-4 mr-2" />
				New Chat
			</Button>
		</div>

		{#if isLoading}
			<div class="flex items-center justify-center py-12">
				<Loader class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else}
			<!-- Quick Stats -->
			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Default Provider</CardTitle>
						<Server class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">
							{defaultProvider || 'Not Set'}
						</div>
						<p class="text-xs text-muted-foreground mt-1">
							{#if defaultProvider}
								Configured and ready
							{:else}
								<a href="/ai/providers" class="text-primary hover:underline">Set up a provider</a>
							{/if}
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Recent Conversations</CardTitle>
						<MessageSquare class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">{recentConversations.length}</div>
						<p class="text-xs text-muted-foreground mt-1">
							<a href="/ai/chat" class="text-primary hover:underline">View all</a>
						</p>
					</CardContent>
				</Card>

				<Card>
					<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle class="text-sm font-medium">Status</CardTitle>
						<CheckCircle2 class="h-4 w-4 text-green-600" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">
							<Badge variant="default" class="bg-green-600">Ready</Badge>
						</div>
						<p class="text-xs text-muted-foreground mt-1">
							AI services operational
						</p>
					</CardContent>
				</Card>
			</div>

			<!-- Quick Actions -->
			<div>
				<h2 class="text-xl font-semibold mb-4">Quick Actions</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each quickActions as action}
						<Card class="hover:shadow-md transition-shadow cursor-pointer" onclick={() => goto(action.url)}>
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
					<div class="flex items-center justify-between mb-4">
						<h2 class="text-xl font-semibold">Recent Conversations</h2>
						<Button variant="ghost" size="sm" onclick={() => goto('/ai/chat')}>
							View All
							<ArrowRight class="h-4 w-4 ml-1" />
						</Button>
					</div>
					<div class="space-y-2">
						{#each recentConversations as conversation}
							<Card class="hover:shadow-md transition-shadow cursor-pointer" onclick={() => goto(`/ai/chat?id=${conversation.id}`)}>
								<CardContent class="p-4">
									<div class="flex items-center justify-between">
										<div class="flex-1">
											<h3 class="font-medium">{conversation.title}</h3>
											<div class="flex items-center gap-2 mt-1">
												<Badge variant="outline">{conversation.provider}</Badge>
												<span class="text-xs text-muted-foreground">
													{new Date(conversation.updated_at).toLocaleDateString()}
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
