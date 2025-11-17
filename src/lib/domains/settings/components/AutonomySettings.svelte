<!--
	Autonomy Settings - Autonomous Action System Configuration
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import Select from '@/lib/components/ui/select.svelte';
	import { autonomyService, type AutonomyLevel, type ApprovalStats } from '@/lib/domains/autonomy';
	import { logger } from '@/lib/domains/shared/services/logger';
	import { toast } from '@/lib/domains/shared/stores/toastStore';
	import { 
		Bot, 
		Shield, 
		CheckCircle2, 
		XCircle, 
		Eye, 
		AlertCircle,
		TrendingUp,
		RefreshCw,
		Activity,
		Zap,
		Settings
	} from '@lucide/svelte';

	const log = logger.createScoped('AutonomySettings');

	let autonomyEnabled = $state(false);
	let autonomyLevel = $state<AutonomyLevel | null>(null);
	let approvalStats = $state<ApprovalStats | null>(null);
	let isLoading = $state(false);
	let isRefreshing = $state(false);

	const levelOptions = [
		{ 
			value: 'observation' as AutonomyLevel, 
			label: 'Observation', 
			description: 'No autonomous actions - suggestions only',
			icon: Eye,
			color: 'text-muted-foreground'
		},
		{ 
			value: 'conservative' as AutonomyLevel, 
			label: 'Conservative', 
			description: 'Safe actions only (read-only, UI updates)',
			icon: Shield,
			color: 'text-blue-500'
		},
		{ 
			value: 'balanced' as AutonomyLevel, 
			label: 'Balanced', 
			description: 'Safe and low-risk actions (common commands)',
			icon: Zap,
			color: 'text-yellow-500'
		},
		{ 
			value: 'aggressive' as AutonomyLevel, 
			label: 'Aggressive', 
			description: 'Most actions (includes medium-risk with high confidence)',
			icon: Activity,
			color: 'text-orange-500'
		},
	];

	const levelDescriptions: Record<AutonomyLevel, string> = {
		observation: 'The system will observe and learn, but never take autonomous actions. All suggestions require manual approval.',
		conservative: 'Only the safest actions will be executed automatically (read-only operations, suggestions, UI updates).',
		balanced: 'Safe actions and common low-risk commands (npm install, git status, etc.) will execute automatically.',
		aggressive: 'Safe, low-risk, and medium-risk actions (with high confidence) will execute automatically. Use with caution.',
	};

	onMount(async () => {
		await loadSettings();
		await loadStats();
	});

	async function loadSettings() {
		isLoading = true;
		try {
			[autonomyEnabled, autonomyLevel] = await Promise.all([
				autonomyService.getEnabled(),
				autonomyService.getLevel(),
			]);
			log.info('Autonomy settings loaded', { autonomyEnabled, autonomyLevel });
		} catch (error) {
			log.error('Failed to load autonomy settings', error);
			toast.error('Failed to load autonomy settings');
		} finally {
			isLoading = false;
		}
	}

	async function loadStats() {
		try {
			approvalStats = await autonomyService.getApprovalStats();
			log.info('Approval stats loaded');
		} catch (error) {
			log.error('Failed to load approval stats', error);
		}
	}

	async function handleToggleEnabled() {
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
			toast.success(`Autonomy level set to ${levelOptions.find(o => o.value === newLevel)?.label}`);
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

	function formatNumber(num: number): string {
		return new Intl.NumberFormat().format(num);
	}

	function formatPercent(num: number): string {
		return `${(num * 100).toFixed(1)}%`;
	}
</script>

<div class="space-y-6">
	<!-- Status Card -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Bot class="w-5 h-5" />
				Autonomous Action System
			</CardTitle>
			<CardDescription>
				Control how the system autonomously executes actions based on learned patterns
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<Label for="autonomy-enabled" class="text-base font-medium">Autonomy Enabled</Label>
					{#if autonomyEnabled}
						<Badge variant="default" class="bg-green-500">
							<CheckCircle2 class="w-3 h-3 mr-1" />
							Active
						</Badge>
					{:else}
						<Badge variant="secondary">
							<XCircle class="w-3 h-3 mr-1" />
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
					<div class="h-10 rounded-md border border-input bg-muted animate-pulse"></div>
				{:else}
					{#each levelOptions as option}
						{#if option.value === autonomyLevel}
							{@const Icon = option.icon}
							<div class="flex items-center gap-2 p-3 border rounded-md bg-muted/50">
								<Icon class="w-5 h-5 {option.color}" />
								<div class="flex-1">
									<div class="font-medium">{option.label}</div>
									<div class="text-sm text-muted-foreground">{option.description}</div>
								</div>
							</div>
						{/if}
					{/each}
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Autonomy Level Control -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Settings class="w-5 h-5" />
				Autonomy Level
			</CardTitle>
			<CardDescription>
				Choose how aggressively the system executes autonomous actions
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label>Select Autonomy Level</Label>
				{#if autonomyLevel !== null}
					<Select
						options={levelOptions.map(o => ({
							value: o.value,
							label: `${o.label} - ${o.description}`,
						}))}
						defaultValue={autonomyLevel}
						onSelect={(value) => value && handleLevelChange(value as AutonomyLevel)}
						disabled={!autonomyEnabled || isLoading}
					/>
				{:else}
					<div class="h-10 rounded-md border border-input bg-muted animate-pulse"></div>
				{/if}
				{#if autonomyLevel !== null}
					<div class="p-3 bg-muted rounded-md text-sm text-muted-foreground">
						{levelDescriptions[autonomyLevel]}
					</div>
				{/if}
			</div>

			{#if !autonomyEnabled}
				<div class="p-3 bg-muted rounded-md text-sm text-muted-foreground">
					Enable autonomy above to adjust level settings.
				</div>
			{/if}

			{#if autonomyLevel === 'aggressive'}
				<div class="p-3 bg-orange-500/10 border border-orange-500/20 rounded-md">
					<div class="flex items-start gap-2">
						<AlertCircle class="w-4 h-4 text-orange-500 mt-0.5 flex-shrink-0" />
						<div class="text-sm">
							<strong class="text-orange-700 dark:text-orange-400">Warning:</strong>
							<span class="text-orange-600 dark:text-orange-300">
								{' '}Aggressive mode allows medium-risk actions to execute automatically. 
								Monitor the system closely and review approval statistics regularly.
							</span>
						</div>
					</div>
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Approval Statistics -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<TrendingUp class="w-5 h-5" />
				Approval Statistics
			</CardTitle>
			<CardDescription>
				Track autonomous action approval rates and patterns
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			{#if approvalStats && Object.keys(approvalStats).length > 0}
				<div class="space-y-3">
					{#each Object.entries(approvalStats) as [actionType, stats]}
						<div class="flex items-center justify-between p-3 border rounded-md">
							<div class="flex-1">
								<div class="font-medium">{actionType}</div>
								<div class="text-sm text-muted-foreground">
									{stats.approved} approved out of {stats.total} requests
								</div>
							</div>
							<div class="text-right">
								<div class="text-lg font-bold">
									{formatPercent(stats.approval_rate)}
								</div>
								<div class="text-xs text-muted-foreground">approval rate</div>
							</div>
						</div>
					{/each}
				</div>

				<Separator />

				<div class="flex justify-end">
					<Button
						variant="ghost"
						size="sm"
						onclick={refreshStats}
						disabled={isRefreshing}
					>
						<RefreshCw class="w-4 h-4 mr-2 {isRefreshing ? 'animate-spin' : ''}" />
						Refresh Stats
					</Button>
				</div>
			{:else}
				<div class="text-center py-8 text-muted-foreground">
					{#if isRefreshing}
						Loading statistics...
					{:else}
						<p>No approval statistics yet.</p>
						<p class="text-xs mt-2">Statistics will appear here as autonomous actions are evaluated.</p>
					{/if}
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Info Card -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Shield class="w-5 h-5" />
				How Autonomy Works
			</CardTitle>
		</CardHeader>
		<CardContent class="space-y-3 text-sm text-muted-foreground">
			<p>
				The autonomous action system learns from your workflow patterns and gradually increases 
				its autonomy based on successful actions.
			</p>
			<ul class="list-disc list-inside space-y-1 ml-2">
				<li><strong>Safe actions:</strong> Read-only operations, suggestions, UI updates</li>
				<li><strong>Low-risk actions:</strong> Common commands like npm install, git status</li>
				<li><strong>Medium-risk actions:</strong> Configuration changes, file creation</li>
				<li><strong>High-risk actions:</strong> Always require approval (file deletion, system changes)</li>
			</ul>
			<p class="pt-2">
				The system tracks approval rates and success rates to determine when actions can be 
				executed autonomously. You can always review and adjust these settings.
			</p>
		</CardContent>
	</Card>
</div>

