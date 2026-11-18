import type { AutonomyLevel } from '$lib/domains/autonomy';
import Shield from '~icons/fa6-solid/shield';
import Eye from '~icons/fa6-solid/eye';
import Activity from '~icons/fa6-solid/wave-square';
import Zap from '~icons/fa6-solid/bolt';

export const levelOptions = [
  {
    value: 'observation' as AutonomyLevel,
    label: 'Observation',
    description: 'No autonomous actions - suggestions only',
    icon: Eye,
    color: 'text-muted-foreground',
  },
  {
    value: 'conservative' as AutonomyLevel,
    label: 'Conservative',
    description: 'Safe actions only (read-only, UI updates)',
    icon: Shield,
    color: 'text-blue-500',
  },
  {
    value: 'balanced' as AutonomyLevel,
    label: 'Balanced',
    description: 'Safe and low-risk actions (common commands)',
    icon: Zap,
    color: 'text-yellow-500',
  },
  {
    value: 'aggressive' as AutonomyLevel,
    label: 'Aggressive',
    description: 'Most actions (includes medium-risk with high confidence)',
    icon: Activity,
    color: 'text-orange-500',
  },
];

export const levelDescriptions: Record<AutonomyLevel, string> = {
  observation:
    'The system will observe and learn, but never take autonomous actions. All suggestions require manual approval.',
  conservative:
    'Only the safest actions will be executed automatically (read-only operations, suggestions, UI updates).',
  balanced:
    'Safe actions and common low-risk commands (npm install, git status, etc.) will execute automatically.',
  aggressive:
    'Safe, low-risk, and medium-risk actions (with high confidence) will execute automatically. Use with caution.',
};
