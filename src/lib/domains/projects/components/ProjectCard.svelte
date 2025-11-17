<!--
	ProjectCard component for displaying project information
	Uses Svelte 5 runes and Tailwind CSS
-->

<script lang="ts">
	import type { Project } from '@/lib/domains/projects/types';
	import { formatRelativeTime, formatBytes } from '@/lib/domains/shared/utils';
	import { Button } from '@/lib/components/ui/button';
	import { Edit, Trash2 } from '@lucide/svelte';

	interface Props {
		project: Project;
		onClick?: (project: Project) => void;
		onEdit?: (project: Project) => void;
		onDelete?: (project: Project) => void;
		showActions?: boolean;
	}

	let { 
		project, 
		onClick = () => {}, 
		onEdit = () => {}, 
		onDelete = () => {},
		showActions = true 
	}: Props = $props();

	const handleClick = () => {
		onClick(project);
	};

	const handleEdit = (e: Event) => {
		e.stopPropagation();
		onEdit(project);
	};

	const handleDelete = (e: Event) => {
		e.stopPropagation();
		onDelete(project);
	};

	// Get framework icon class
	const getFrameworkIconClass = (framework: string | null | undefined): string => {
		if (!framework) return 'devicon-folder-plain';
		
		const icons: Record<string, string> = {
			'React': 'devicon-react-original',
			'Vue': 'devicon-vuejs-plain',
			'Angular': 'devicon-angularjs-plain',
			'Svelte': 'devicon-svelte-plain',
			'Next.js': 'devicon-nextjs-plain',
			'Nuxt': 'devicon-nuxtjs-plain',
			'Node.js': 'devicon-nodejs-plain',
			'Express': 'devicon-express-original',
			'FastAPI': 'devicon-fastapi-plain',
			'Django': 'devicon-django-plain',
			'Flask': 'devicon-flask-plain',
			'Laravel': 'devicon-laravel-plain',
			'Spring': 'devicon-spring-plain',
			'ASP.NET': 'devicon-dotnetcore-plain',
			'Rails': 'devicon-rails-plain',
			'Flutter': 'devicon-flutter-plain',
			'React Native': 'devicon-react-original',
			'Ionic': 'devicon-ionic-original',
			'Electron': 'devicon-electron-original',
			'Tauri': 'devicon-rust-plain',
			'Python': 'devicon-python-plain',
			'Java': 'devicon-java-plain',
			'Go': 'devicon-go-plain',
			'Rust': 'devicon-rust-plain',
			'PHP': 'devicon-php-plain',
			'Ruby': 'devicon-ruby-plain',
			'Swift': 'devicon-swift-plain',
			'TypeScript': 'devicon-typescript-plain',
			'JavaScript': 'devicon-javascript-plain'
		};
		return icons[framework] || 'devicon-folder-plain';
	};

	// Get framework color
	const getFrameworkColor = (framework: string | null | undefined): string => {
		if (!framework) return 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200';
		
		const colors: Record<string, string> = {
			'React': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Vue': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Angular': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Svelte': 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
			'Next.js': 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
			'Node.js': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Express': 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
			'FastAPI': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Django': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'Flask': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Laravel': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Spring': 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
			'ASP.NET': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Rails': 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
			'Flutter': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'React Native': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Electron': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
			'Tauri': 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200'
		};
		return colors[framework] || 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200';
	};


</script>

<div 
	class="card card-hover cursor-pointer group"
	onclick={handleClick}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && handleClick()}
>
	<div class="flex items-start justify-between">
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 mb-2">
				<i class="text-lg {getFrameworkIconClass(project.framework)}"></i>
				<h3 class="text-lg font-semibold text-neutral-900 dark:text-neutral-100 truncate">
					{project.name}
				</h3>
			</div>
			
			{#if project.description}
				<p class="text-sm text-neutral-600 dark:text-neutral-400 mb-3 line-clamp-2">
					{project.description}
				</p>
			{/if}

			<div class="flex items-center gap-2 mb-3">
				{#if project.framework}
					<span class="badge {getFrameworkColor(project.framework)}">
						{project.framework}
					</span>
				{/if}
				{#if project.metadata.gitInfo?.branch}
					<span class="badge badge-neutral">
						{project.metadata.gitInfo.branch}
					</span>
				{/if}
			</div>

			<div class="flex items-center gap-4 text-xs text-neutral-500 dark:text-neutral-400">
				{#if project.metadata.lastOpened}
					<span>Last opened {formatRelativeTime(project.metadata.lastOpened)}</span>
				{/if}
				<span>{formatBytes(project.metadata.size)}</span>
				<span>{project.metadata.fileCount} files</span>
			</div>
		</div>

		{#if showActions}
			<div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
				<Button
					variant="ghost"
					size="sm"
					onclick={handleEdit}
					class="h-8 w-8 p-0"
					title="Edit project"
					aria-label="Edit project"
				>
					<Edit class="w-4 h-4" />
				</Button>
				<Button
					variant="ghost"
					size="sm"
					onclick={handleDelete}
					class="h-8 w-8 p-0 text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
					title="Delete project"
					aria-label="Delete project"
				>
					<Trash2 class="w-4 h-4" />
				</Button>
			</div>
		{/if}
	</div>
</div>
