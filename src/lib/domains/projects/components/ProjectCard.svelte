<!--
	ProjectCard component for displaying project information
	Uses Svelte 5 runes and Tailwind CSS
-->

<script lang="ts">
	import type { Project } from '@/lib/domains/projects/types';
	import { formatRelativeTime, formatBytes } from '@/lib/domains/shared/utils';

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
				<button
					class="btn-icon"
					onclick={handleEdit}
					title="Edit project"
					aria-label="Edit project"
				>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
					</svg>
				</button>
				<button
					class="btn-icon text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
					onclick={handleDelete}
					title="Delete project"
					aria-label="Delete project"
				>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
					</svg>
				</button>
			</div>
		{/if}
	</div>
</div>
