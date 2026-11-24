/** Human copy for the capability ids in the Rust registry (`rpc.rs`). The registry
 * carries ids only, so this is the frontend's side of that contract; an unknown id
 * falls back to itself. */
const LABELS: Record<string, { label: string; hint: string }> = {
  actions: { label: 'Quick actions', hint: 'Run your saved actions' },
  agents: { label: 'Agents', hint: 'See agent runs and models' },
  'agents.exec': { label: 'Run agents', hint: 'Start agents and answer their permission prompts' },
  'agents.history': { label: 'Agent history', hint: 'Read full transcripts of past agent runs' },
  coder: { label: 'Coder (read)', hint: 'View coder sessions and status' },
  'coder.exec': { label: 'Coder (run)', hint: 'Write files, run processes and commit' },
  dashboard: { label: 'Dashboard', hint: 'See the dashboard overview' },
  github: { label: 'GitHub (read)', hint: 'View repositories, issues and runs' },
  'github.write': { label: 'GitHub (write)', hint: 'Edit issues and trigger workflows' },
  'n8n.run': { label: 'Run n8n workflows', hint: 'Trigger workflows, which can run anything' },
  pipelines: { label: 'Pipelines', hint: 'View and manage pipelines' },
  projects: { label: 'Projects', hint: 'Browse your projects' },
  'remote.input': { label: 'Remote input', hint: 'Send taps and keystrokes to this computer' },
  sysinfo: { label: 'System info', hint: 'See CPU, memory and disk usage' },
  tasks: { label: 'Tasks', hint: 'View and edit the task board' },
  terminal: { label: 'Terminal (view)', hint: 'See terminal sessions' },
  'terminal.exec': { label: 'Terminal (run)', hint: 'Run commands on this computer' },
};

export function capabilityLabel(id: string): { label: string; hint: string } {
  return LABELS[id] ?? { label: id, hint: '' };
}
