import type { MLIntensity } from '$lib/domains/learning';

export const intensityOptions = [
  {
    value: 'fast' as MLIntensity,
    label: 'Fast',
    description: 'Minimal learning, basic patterns only',
  },
  {
    value: 'light' as MLIntensity,
    label: 'Light',
    description: 'Limited analysis, basic suggestions',
  },
  {
    value: 'medium' as MLIntensity,
    label: 'Medium',
    description: 'Full analysis, contextual suggestions',
  },
  {
    value: 'deep' as MLIntensity,
    label: 'Deep',
    description: 'Advanced analysis, predictive actions',
  },
];

export const intensityDescriptions: Record<MLIntensity, string> = {
  fast: 'Processes in large batches every 60 seconds. Minimal resource usage.',
  light: 'Processes in medium batches every 30 seconds. Low resource usage.',
  medium: 'Processes in small batches every 15 seconds. Moderate resource usage.',
  deep: 'Processes in very small batches every 5 seconds. Higher resource usage.',
};
