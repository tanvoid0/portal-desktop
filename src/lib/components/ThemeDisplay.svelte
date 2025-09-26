<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';
	import { Switch } from '$lib/components/ui/switch';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Progress } from '$lib/components/ui/progress';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import {
		AlertTriangle,
		CheckCircle,
		Info,
		Settings,
		User,
		Mail,
		Bell,
		Download
	} from '@lucide/svelte';

	// State for interactive components
	let switchValue = $state(false);
	let checkboxValue = $state(false);
	let inputValue = $state('');
	let progressValue = $state(65);

	function handleButtonClick() {
		alert('Button clicked! UI components are working correctly.');
	}
</script>

<div class="container mx-auto space-y-8 p-6">
	<div class="space-y-2 text-center">
		<h1 class="text-4xl font-bold">Theme Display Widget</h1>
		<p class="text-muted-foreground">Testing UI Components & Theme System</p>
	</div>

	<Tabs value="components" class="w-full">
		<TabsList class="grid w-full grid-cols-3">
			<TabsTrigger value="components">Components</TabsTrigger>
			<TabsTrigger value="forms">Forms</TabsTrigger>
			<TabsTrigger value="feedback">Feedback</TabsTrigger>
		</TabsList>

		<TabsContent value="components" class="space-y-6">
			<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
				<!-- Buttons Card -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Settings class="h-5 w-5" />
							Buttons
						</CardTitle>
						<CardDescription>Various button styles and states</CardDescription>
					</CardHeader>
					<CardContent class="space-y-3">
						<div class="flex flex-wrap gap-2">
							<Button onclick={handleButtonClick}>Default</Button>
							<Button variant="secondary">Secondary</Button>
							<Button variant="destructive">Destructive</Button>
							<Button variant="outline">Outline</Button>
							<Button variant="ghost">Ghost</Button>
							<Button variant="link">Link</Button>
						</div>
						<div class="flex flex-wrap gap-2">
							<Button size="sm">Small</Button>
							<Button size="default">Default</Button>
							<Button size="lg">Large</Button>
						</div>
					</CardContent>
				</Card>

				<!-- Badges Card -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<CheckCircle class="h-5 w-5" />
							Badges
						</CardTitle>
						<CardDescription>Status indicators and labels</CardDescription>
					</CardHeader>
					<CardContent class="space-y-3">
						<div class="flex flex-wrap gap-2">
							<Badge>Default</Badge>
							<Badge variant="secondary">Secondary</Badge>
							<Badge variant="destructive">Error</Badge>
							<Badge variant="outline">Outline</Badge>
						</div>
						<div class="flex flex-wrap gap-2">
							<Badge class="bg-green-500">Success</Badge>
							<Badge class="bg-yellow-500">Warning</Badge>
							<Badge class="bg-blue-500">Info</Badge>
						</div>
					</CardContent>
				</Card>

				<!-- Progress Card -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Download class="h-5 w-5" />
							Progress
						</CardTitle>
						<CardDescription>Loading and progress indicators</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<div class="flex justify-between text-sm">
								<span>Progress</span>
								<span>{progressValue}%</span>
							</div>
							<Progress value={progressValue} class="w-full" />
						</div>
						<div class="flex gap-2">
							<Button size="sm" onclick={() => (progressValue = Math.max(0, progressValue - 10))}>
								-10%
							</Button>
							<Button size="sm" onclick={() => (progressValue = Math.min(100, progressValue + 10))}>
								+10%
							</Button>
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>

		<TabsContent value="forms" class="space-y-6">
			<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
				<!-- Form Controls Card -->
				<Card>
					<CardHeader>
						<CardTitle>Form Controls</CardTitle>
						<CardDescription>Interactive form elements</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label for="test-input">Input Field</Label>
							<Input id="test-input" bind:value={inputValue} placeholder="Type something..." />
							<p class="text-sm text-muted-foreground">Value: {inputValue}</p>
						</div>

						<Separator />

						<div class="space-y-4">
							<div class="flex items-center space-x-2">
								<Switch bind:checked={switchValue} />
								<Label>Switch: {switchValue ? 'On' : 'Off'}</Label>
							</div>

							<div class="flex items-center space-x-2">
								<Checkbox bind:checked={checkboxValue} />
								<Label>Checkbox: {checkboxValue ? 'Checked' : 'Unchecked'}</Label>
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Icons Card -->
				<Card>
					<CardHeader>
						<CardTitle>Icons</CardTitle>
						<CardDescription>Lucide icons integration</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="grid grid-cols-4 gap-4">
							<div class="flex flex-col items-center space-y-2 rounded-lg bg-muted p-3">
								<User class="h-6 w-6" />
								<span class="text-xs">User</span>
							</div>
							<div class="flex flex-col items-center space-y-2 rounded-lg bg-muted p-3">
								<Settings class="h-6 w-6" />
								<span class="text-xs">Settings</span>
							</div>
							<div class="flex flex-col items-center space-y-2 rounded-lg bg-muted p-3">
								<Mail class="h-6 w-6" />
								<span class="text-xs">Mail</span>
							</div>
							<div class="flex flex-col items-center space-y-2 rounded-lg bg-muted p-3">
								<Bell class="h-6 w-6" />
								<span class="text-xs">Bell</span>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>

		<TabsContent value="feedback" class="space-y-6">
			<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
				<!-- Alerts Card -->
				<Card>
					<CardHeader>
						<CardTitle>Alert Messages</CardTitle>
						<CardDescription>Different alert types</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<Alert>
							<Info class="h-4 w-4" />
							<AlertDescription>This is an informational alert message.</AlertDescription>
						</Alert>

						<Alert variant="destructive">
							<AlertTriangle class="h-4 w-4" />
							<AlertDescription>This is a destructive alert message.</AlertDescription>
						</Alert>
					</CardContent>
				</Card>

				<!-- Theme Colors Card -->
				<Card>
					<CardHeader>
						<CardTitle>Theme Colors</CardTitle>
						<CardDescription>CSS custom properties</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="grid grid-cols-2 gap-3">
							<div class="rounded-lg bg-primary p-3 text-center text-primary-foreground">
								<div class="font-semibold">Primary</div>
								<div class="text-xs opacity-90">hsl(var(--primary))</div>
							</div>
							<div class="rounded-lg bg-secondary p-3 text-center text-secondary-foreground">
								<div class="font-semibold">Secondary</div>
								<div class="text-xs opacity-90">hsl(var(--secondary))</div>
							</div>
							<div class="rounded-lg bg-muted p-3 text-center text-muted-foreground">
								<div class="font-semibold">Muted</div>
								<div class="text-xs opacity-90">hsl(var(--muted))</div>
							</div>
							<div class="rounded-lg bg-accent p-3 text-center text-accent-foreground">
								<div class="font-semibold">Accent</div>
								<div class="text-xs opacity-90">hsl(var(--accent))</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>
	</Tabs>

	<Card class="mt-8">
		<CardHeader>
			<CardTitle>Component Status</CardTitle>
			<CardDescription>Verification that all UI components are working correctly</CardDescription>
		</CardHeader>
		<CardContent>
			<div class="grid grid-cols-2 gap-4 md:grid-cols-4">
				<div class="flex items-center space-x-2 rounded-lg bg-green-50 p-3 dark:bg-green-950">
					<CheckCircle class="h-5 w-5 text-green-600" />
					<span class="text-sm font-medium">Buttons</span>
				</div>
				<div class="flex items-center space-x-2 rounded-lg bg-green-50 p-3 dark:bg-green-950">
					<CheckCircle class="h-5 w-5 text-green-600" />
					<span class="text-sm font-medium">Cards</span>
				</div>
				<div class="flex items-center space-x-2 rounded-lg bg-green-50 p-3 dark:bg-green-950">
					<CheckCircle class="h-5 w-5 text-green-600" />
					<span class="text-sm font-medium">Forms</span>
				</div>
				<div class="flex items-center space-x-2 rounded-lg bg-green-50 p-3 dark:bg-green-950">
					<CheckCircle class="h-5 w-5 text-green-600" />
					<span class="text-sm font-medium">Icons</span>
				</div>
			</div>
		</CardContent>
	</Card>
</div>
