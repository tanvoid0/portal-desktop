/** Hand a stuck repository to the coder, with the rules it has to keep. */
import { homeDir } from '@tauri-apps/api/path';
import { openCoderSession } from '$lib/domains/coder/utils/launch';
import { toast } from '$lib/utils/toast';
import type { SyncProjectResult } from '../generated/SyncProjectResult';
import { commonDir, isTooBroad } from './commonDir';
import { openSteps } from './syncTriage';

const RULES = [
  'Inspect each repository (git status, git log) first, then resolve these.',
  'Do not push, force, reset --hard, rewrite published history or delete files without asking me.',
  'Never commit .env files, keys or other secrets.',
  'Show me what you are going to commit or merge before you do it.',
];

/** Env steps are vault work, which the coder cannot do. */
export const coderProblems = (p: SyncProjectResult) =>
  openSteps(p.steps).filter((s) => s.step !== 'env');

const detail = (p: SyncProjectResult) =>
  coderProblems(p).map((s) => `- ${s.step}: ${s.detail ?? s.status}`);

/** The coder pre-fills this and waits for the user to send it. */
export function openInCoder(project: SyncProjectResult) {
  openCoderSession({
    workspaceRoot: project.path,
    projectId: String(project.id),
    prompt: [
      `"Sync all projects" could not finish syncing "${project.name}":`,
      ...detail(project),
      '',
      ...RULES,
    ].join('\n'),
  });
}

/** One session rooted above every stuck repo, working through them in turn. */
export async function openAllInCoder(stuck: SyncProjectResult[]) {
  const root = commonDir(stuck.map((p) => p.path));
  // No home folder to compare against means no shared workspace either.
  const home = await homeDir().catch(() => null);
  if (!root || home === null || isTooBroad(root, home)) {
    toast.error('These projects share no parent folder. Open them one by one.');
    return;
  }
  openCoderSession({
    workspaceRoot: root,
    prompt: [
      `"Sync all projects" could not finish syncing ${stuck.length} projects. Work through them one at a time:`,
      ...stuck.flatMap((p) => ['', `${p.name} (${p.path})`, ...detail(p)]),
      '',
      ...RULES,
    ].join('\n'),
  });
}
