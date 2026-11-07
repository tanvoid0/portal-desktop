/**
 * Status presentation for script executions, shared by the run panel and the history list.
 */
import Square from '~icons/fa6-solid/square';
import Clock from '~icons/fa6-solid/clock';
import CheckCircle from '~icons/fa6-solid/circle-check';
import XCircle from '~icons/fa6-solid/circle-xmark';
import Loader2 from '~icons/fa6-solid/spinner';

export function getStatusIcon(status: string) {
  switch (status) {
    case 'pending':
      return Clock;
    case 'running':
      return Loader2;
    case 'success':
      return CheckCircle;
    case 'failed':
      return XCircle;
    case 'cancelled':
      return Square;
    default:
      return Clock;
  }
}

export function getStatusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'text-yellow-500';
    case 'running':
      return 'text-blue-500 animate-spin';
    case 'success':
      return 'text-green-500';
    case 'failed':
      return 'text-red-500';
    case 'cancelled':
      return 'text-gray-500';
    default:
      return 'text-muted-foreground';
  }
}

export function formatDuration(start: string, end: string | null): string {
  const startDate = new Date(start);
  const endDate = end ? new Date(end) : new Date();
  const diff = endDate.getTime() - startDate.getTime();

  if (diff < 1000) return `${diff}ms`;
  if (diff < 60000) return `${Math.floor(diff / 1000)}s`;
  return `${Math.floor(diff / 60000)}m ${Math.floor((diff % 60000) / 1000)}s`;
}
