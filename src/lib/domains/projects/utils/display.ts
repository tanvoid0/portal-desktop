/**
 * Display helpers for project cards and lists
 */

import type { Project } from "$lib/domains/projects/types";

const FRAMEWORK_ICONS: Record<string, string> = {
  React: "devicon-react-original colored",
  Vue: "devicon-vuejs-plain colored",
  Angular: "devicon-angularjs-plain colored",
  Svelte: "devicon-svelte-plain colored",
  "Next.js": "devicon-nextjs-plain colored",
  Nuxt: "devicon-nuxtjs-plain colored",
  "Node.js": "devicon-nodejs-plain colored",
  Express: "devicon-express-original colored",
  FastAPI: "devicon-fastapi-plain colored",
  Django: "devicon-django-plain colored",
  Flask: "devicon-flask-plain colored",
  Laravel: "devicon-laravel-plain colored",
  Spring: "devicon-spring-plain colored",
  "ASP.NET": "devicon-dotnetcore-plain colored",
  Rails: "devicon-rails-plain colored",
  Flutter: "devicon-flutter-plain colored",
  "React Native": "devicon-react-original colored",
  Ionic: "devicon-ionic-original colored",
  Electron: "devicon-electron-original colored",
  Tauri: "devicon-rust-plain colored",
  Python: "devicon-python-plain colored",
  Java: "devicon-java-plain colored",
  Go: "devicon-go-plain colored",
  Rust: "devicon-rust-plain colored",
  PHP: "devicon-php-plain colored",
  Ruby: "devicon-ruby-plain colored",
  Swift: "devicon-swift-plain colored",
  TypeScript: "devicon-typescript-plain colored",
  JavaScript: "devicon-javascript-plain colored",
};

const FRAMEWORK_COLORS: Record<string, string> = {
  React: "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200",
  Vue: "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
  Angular: "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200",
  Svelte:
    "bg-orange-100 text-orange-800 dark:bg-orange-900/50 dark:text-orange-200",
  "Next.js": "bg-gray-100 text-gray-800 dark:bg-gray-900/50 dark:text-gray-200",
  "Node.js":
    "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
  Express: "bg-gray-100 text-gray-800 dark:bg-gray-900/50 dark:text-gray-200",
  FastAPI:
    "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
  Django:
    "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
  Flask: "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200",
  Laravel: "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200",
  Spring:
    "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
  "ASP.NET":
    "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200",
  Rails: "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200",
  Flutter: "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200",
  "React Native":
    "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200",
  Electron: "bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200",
  Tauri:
    "bg-orange-100 text-orange-800 dark:bg-orange-900/50 dark:text-orange-200",
};

const FRAMEWORK_ICON_BACKGROUNDS: Record<string, string> = {
  React: "bg-blue-500/10",
  Vue: "bg-green-500/10",
  Angular: "bg-red-500/10",
  Svelte: "bg-orange-500/10",
  "Next.js": "bg-gray-500/10",
  "Node.js": "bg-green-500/10",
  Express: "bg-gray-500/10",
  FastAPI: "bg-green-500/10",
  Django: "bg-green-500/10",
  Flask: "bg-red-500/10",
  Laravel: "bg-red-500/10",
  Spring: "bg-green-500/10",
  "ASP.NET": "bg-blue-500/10",
  Rails: "bg-red-500/10",
  Flutter: "bg-blue-500/10",
  "React Native": "bg-blue-500/10",
  Electron: "bg-blue-500/10",
  Tauri: "bg-orange-500/10",
  Python: "bg-yellow-500/10",
  Java: "bg-orange-500/10",
  Go: "bg-cyan-500/10",
  Rust: "bg-orange-500/10",
  TypeScript: "bg-blue-500/10",
  JavaScript: "bg-yellow-500/10",
};

const PACKAGE_MANAGER_ICONS: Record<string, string> = {
  npm: "devicon-npm-original-wordmark colored",
  yarn: "devicon-yarn-plain colored",
  pnpm: "devicon-pnpm-plain colored",
  pip: "devicon-python-plain colored",
  cargo: "devicon-rust-plain colored",
  go: "devicon-go-plain colored",
  composer: "devicon-composer-plain colored",
  bun: "devicon-bun-plain colored",
  maven: "devicon-maven-plain colored",
  gradle: "devicon-gradle-plain colored",
};

const DEFAULT_BADGE_COLOR =
  "bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200";
const DEFAULT_ICON_BG = "bg-primary/10";

type ProjectWithBackendFields = Project & {
  framework?: string;
  package_manager?: string;
};

export function getProjectFramework(
  project: Project,
): string | undefined {
  const extended = project as ProjectWithBackendFields;
  return project.metadata?.framework ?? extended.framework;
}

export function getProjectPackageManager(
  project: Project,
): string | undefined {
  const extended = project as ProjectWithBackendFields;
  return (
    project.metadata?.dependencies?.packageManager ??
    extended.package_manager
  );
}

export function getProjectGitBranch(project: Project): string | undefined {
  return project.metadata?.gitInfo?.branch ?? project.git_branch;
}

export function getProjectGitCommit(project: Project): string | undefined {
  return project.metadata?.gitInfo?.commit ?? project.git_commit;
}

export function getProjectFileCount(project: Project): number {
  return project.file_count ?? project.metadata?.fileCount ?? 0;
}

export function getProjectSize(project: Project): number {
  return project.size ?? project.metadata?.size ?? 0;
}

export function getFrameworkIcon(
  framework: string | null | undefined,
): string {
  if (!framework) return "devicon-folder-plain";
  return FRAMEWORK_ICONS[framework] ?? "devicon-folder-plain";
}

export function getFrameworkColor(
  framework: string | null | undefined,
): string {
  if (!framework) return DEFAULT_BADGE_COLOR;
  return FRAMEWORK_COLORS[framework] ?? DEFAULT_BADGE_COLOR;
}

export function getFrameworkIconBackground(
  framework: string | null | undefined,
): string {
  if (!framework) return DEFAULT_ICON_BG;
  return FRAMEWORK_ICON_BACKGROUNDS[framework] ?? DEFAULT_ICON_BG;
}

export function getPackageManagerIcon(
  packageManager: string | null | undefined,
): string | undefined {
  if (!packageManager) return undefined;
  return (
    PACKAGE_MANAGER_ICONS[packageManager.toLowerCase()] ??
    "devicon-devicon-plain"
  );
}

export function getStatusColor(status: string): string {
  switch (status) {
    case "active":
      return "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200";
    case "archived":
      return "bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-200";
    case "deleted":
      return "bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-200";
    default:
      return DEFAULT_BADGE_COLOR;
  }
}

export function truncatePath(path: string, maxLength = 42): string {
  if (path.length <= maxLength) return path;
  const parts = path.split(/[/\\]/);
  if (parts.length <= 2) {
    return `…${path.slice(-maxLength + 1)}`;
  }
  return `…/${parts.slice(-2).join("/")}`;
}

export function shortCommitHash(commit: string | undefined): string | undefined {
  if (!commit) return undefined;
  return commit.length > 7 ? commit.slice(0, 7) : commit;
}
