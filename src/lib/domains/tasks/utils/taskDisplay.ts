import type { Task } from '../types';

export function getTaskStatusColor(status: string): string {
  switch (status) {
    case 'completed':
      return 'text-green-500';
    case 'in-progress':
      return 'text-blue-500';
    case 'cancelled':
      return 'text-red-500';
    default:
      return 'text-gray-400';
  }
}

export function getStatusBadgeColor(status: string): string {
  switch (status) {
    case 'completed':
      return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
    case 'in-progress':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
    case 'cancelled':
      return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300';
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
  }
}

export function getPriorityColor(priority: string): string {
  switch (priority) {
    case 'high':
      return 'text-red-500';
    case 'medium':
      return 'text-yellow-500';
    case 'low':
      return 'text-green-500';
    default:
      return 'text-gray-400';
  }
}

export function getTaskIcon(task: Task): string {
  switch (task.status) {
    case 'completed':
      return 'mdi:check-circle';
    case 'in-progress':
      return 'mdi:progress-clock';
    case 'cancelled':
      return 'mdi:cancel';
    default:
      return 'mdi:circle-outline';
  }
}

const isOpen = (t: Task) => t.status !== 'completed' && t.status !== 'cancelled';

export function isOverdue(t: Task): boolean {
  return !!t.dueDate && t.dueDate < new Date() && isOpen(t);
}

export function isDueToday(t: Task): boolean {
  if (!t.dueDate || !isOpen(t)) return false;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const tomorrow = new Date(today);
  tomorrow.setDate(tomorrow.getDate() + 1);
  return t.dueDate >= today && t.dueDate < tomorrow;
}

export function isDueThisWeek(t: Task): boolean {
  if (!t.dueDate || !isOpen(t)) return false;
  const now = new Date();
  const weekFromNow = new Date(now);
  weekFromNow.setDate(weekFromNow.getDate() + 7);
  return t.dueDate >= now && t.dueDate <= weekFromNow;
}

export function formatMinutes(minutes?: number): string {
  if (!minutes) return '';
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? (m > 0 ? `${h}h ${m}m` : `${h}h`) : `${m}m`;
}

/** Colour/icon strategy so non-task lists (GitHub issues) can reuse the board and list. */
export interface TaskDisplay {
  statusColor: (status: string) => string;
  statusBadge: (status: string) => string;
  priorityColor: (priority: string) => string;
  icon: (task: Task) => string;
}

export const defaultDisplay: TaskDisplay = {
  statusColor: getTaskStatusColor,
  statusBadge: getStatusBadgeColor,
  priorityColor: getPriorityColor,
  icon: getTaskIcon,
};
