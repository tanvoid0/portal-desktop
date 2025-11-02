<!--
	Documents Page - Workspace documentation and notes
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { logger } from '$lib/domains/shared';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { FileText, Plus, Search, Folder, BookOpen, Edit } from 'lucide-svelte';
</script>

<svelte:head>
	<title>Documents - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-3xl font-bold tracking-tight flex items-center gap-2">
					<FileText class="h-8 w-8" />
					Documents
				</h1>
				<p class="text-muted-foreground">
					Workspace documentation, notes, and knowledge base
				</p>
			</div>
			<div class="flex gap-2">
				<Button variant="outline">
					<Search class="h-4 w-4 mr-2" />
					Search
				</Button>
				<Button>
					<Plus class="h-4 w-4 mr-2" />
					New Document
				</Button>
			</div>
		</div>

		<!-- Coming Soon Card -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<BookOpen class="h-5 w-5" />
					Document Manager
				</CardTitle>
				<CardDescription>
					Create, edit, and organize your workspace documentation
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="space-y-4">
					<div class="flex items-center gap-4">
						<Badge variant="outline" class="flex items-center gap-1">
							<FileText class="h-3 w-3" />
							Documents: 0
						</Badge>
						<Badge variant="outline" class="flex items-center gap-1">
							<Folder class="h-3 w-3" />
							Folders: 0
						</Badge>
					</div>
					
					<div class="text-center py-8">
						<Edit class="h-12 w-12 mx-auto text-muted-foreground mb-4" />
						<h3 class="text-lg font-semibold mb-2">Document Manager Coming Soon</h3>
						<p class="text-muted-foreground mb-4">
							Markdown editor with live preview and full-text search
						</p>
						<div class="space-y-2 text-sm text-muted-foreground">
							<p>• Markdown editor with live preview</p>
							<p>• Full-text search across documents</p>
							<p>• Document versioning and history</p>
							<p>• Tag-based organization</p>
							<p>• Export to PDF/HTML</p>
							<p>• Template system</p>
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
