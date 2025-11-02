/**
 * Duration formatting utilities for command execution times
 */

export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) {
    return `${(ms / 1000).toFixed(1)}s`;
  }
  
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  
  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  
  return `${hours}h ${remainingMinutes}m`;
}

export function getDurationColor(duration: number): string {
  if (duration < 5000) {
    return 'text-green-400'; // Fast: < 5s
  } else if (duration < 30000) {
    return 'text-yellow-400'; // Medium: 5s - 30s
  } else {
    return 'text-orange-400'; // Slow: > 30s
  }
}

export function getDurationBadgeVariant(duration: number): 'default' | 'secondary' | 'destructive' | 'outline' {
  if (duration < 5000) {
    return 'default';
  } else if (duration < 30000) {
    return 'secondary';
  } else {
    return 'destructive';
  }
}

export function calculateAverageDuration(entries: Array<{ duration?: number }>): number {
  const durations = entries
    .map(entry => entry.duration)
    .filter((duration): duration is number => duration !== undefined && duration > 0);
  
  if (durations.length === 0) return 0;
  
  return durations.reduce((sum, duration) => sum + duration, 0) / durations.length;
}
