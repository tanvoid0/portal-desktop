export function getSeverityColor(severity: string): string {
  const s = severity?.toUpperCase();
  switch (s) {
    case 'ERROR':
    case 'FATAL':
    case 'CRITICAL':
      return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200 border-l-4 border-red-400';
    case 'WARN':
    case 'WARNING':
      return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 border-l-4 border-yellow-400';
    case 'INFO':
    case 'INFORMATION':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 border-l-4 border-blue-400';
    case 'DEBUG':
    case 'TRACE':
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200 border-l-4 border-gray-400';
    default:
      return 'bg-slate-100 text-slate-800 dark:bg-slate-900 dark:text-slate-200 border-l-4 border-slate-400';
  }
}

export function getSeverityBadgeVariant(
  severity: string
): 'default' | 'secondary' | 'destructive' | 'outline' {
  const s = severity?.toUpperCase();
  switch (s) {
    case 'ERROR':
    case 'FATAL':
    case 'CRITICAL':
      return 'destructive';
    case 'WARN':
    case 'WARNING':
      return 'outline';
    default:
      return 'default';
  }
}
