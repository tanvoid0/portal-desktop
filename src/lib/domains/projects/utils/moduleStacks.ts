/**
 * Labels and icons for module stacks, and the summaries the list and detail pages
 * show by default. Stack ids are what the Rust scanner emits (`detect.rs`).
 */

import type { ProjectModule } from '$lib/domains/projects/types';

const STACKS: Record<string, { label: string; icon?: string }> = {
  flutter: { label: 'Flutter', icon: 'devicon-flutter-plain colored' },
  dart: { label: 'Dart', icon: 'devicon-dart-plain colored' },
  nestjs: { label: 'NestJS', icon: 'devicon-nestjs-original colored' },
  android: { label: 'Android', icon: 'devicon-android-plain colored' },
  gradle: { label: 'Gradle', icon: 'devicon-gradle-plain colored' },
  rust: { label: 'Rust', icon: 'devicon-rust-plain colored' },
  node: { label: 'Node', icon: 'devicon-nodejs-plain colored' },
  nextjs: { label: 'Next.js', icon: 'devicon-nextjs-plain colored' },
  sveltekit: { label: 'SvelteKit', icon: 'devicon-svelte-plain colored' },
  svelte: { label: 'Svelte', icon: 'devicon-svelte-plain colored' },
  nuxt: { label: 'Nuxt', icon: 'devicon-nuxtjs-plain colored' },
  vue: { label: 'Vue', icon: 'devicon-vuejs-plain colored' },
  react: { label: 'React', icon: 'devicon-react-original colored' },
  angular: { label: 'Angular', icon: 'devicon-angularjs-plain colored' },
  tauri: { label: 'Tauri', icon: 'devicon-tauri-original colored' },
  go: { label: 'Go', icon: 'devicon-go-original-wordmark colored' },
  python: { label: 'Python', icon: 'devicon-python-plain colored' },
  java: { label: 'Java', icon: 'devicon-java-plain colored' },
  php: { label: 'PHP', icon: 'devicon-php-plain colored' },
  ruby: { label: 'Ruby', icon: 'devicon-ruby-plain colored' },
  dotnet: { label: '.NET', icon: 'devicon-dotnetcore-plain colored' },
};

export function stackLabel(stack: string): string {
  return STACKS[stack]?.label ?? stack;
}

export function stackIcon(stack: string): string | undefined {
  return STACKS[stack]?.icon;
}

export function groupByProject(modules: ProjectModule[]): Map<number, ProjectModule[]> {
  const byProject = new Map<number, ProjectModule[]>();
  for (const m of modules) {
    const list = byProject.get(m.project_id);
    if (list) list.push(m);
    else byProject.set(m.project_id, [m]);
  }
  return byProject;
}

export interface StackCount {
  stack: string;
  label: string;
  count: number;
}

/** Stacks present in `modules`, most common first. Ties break alphabetically. */
export function summarizeStacks(modules: ProjectModule[]): StackCount[] {
  const counts = new Map<string, number>();
  for (const m of modules) counts.set(m.stack, (counts.get(m.stack) ?? 0) + 1);
  return [...counts]
    .map(([stack, count]) => ({ stack, label: stackLabel(stack), count }))
    .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label));
}

/** "9 Flutter · 1 Android": the one-line answer to what is inside. */
export function describeModules(modules: ProjectModule[], limit = 3): string {
  const stacks = summarizeStacks(modules);
  const shown = stacks.slice(0, limit).map((s) => `${s.count} ${s.label}`);
  const hidden = stacks.length - limit;
  return hidden > 0 ? `${shown.join(' · ')} · +${hidden}` : shown.join(' · ');
}
