import { goto } from '$app/navigation';
import { terminalActions } from '$lib/domains/terminal/stores/terminalStore';
import { defaultTerminalConfig } from '$lib/domains/terminal/config/defaultTerminalConfig';
import { TERMINAL_ICONS } from '$lib/domains/terminal/utils/shellIcons';

/**
 * Resumes a Claude Code CLI session (the `session:<id>` tag a task picks up
 * while a crew agent is working it) in a new terminal tab. Portal's own
 * Agents page only knows about runs it started itself (keyed by its own
 * run_id), not sessions claimed via the task board, so a terminal running
 * `claude --resume` is the practical way to jump back into the real session.
 */
export function openClaudeSession(sessionId: string, opts: { title: string; cwd?: string }): void {
  const tabId = terminalActions.createTabWithProcess({
    title: opts.title,
    workingDirectory: opts.cwd || defaultTerminalConfig.workingDirectory,
    shell: `claude --resume ${sessionId}`,
    icon: TERMINAL_ICONS.default,
    resourceName: 'claude-session',
    resourceId: sessionId,
  });
  terminalActions.setActiveTab(tabId);
  void goto('/terminal');
}
