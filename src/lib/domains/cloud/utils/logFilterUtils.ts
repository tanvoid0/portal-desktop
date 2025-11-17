// Log filtering utilities for cloud domain
// Provides search and filtering capabilities for Kubernetes pod logs

import type { K8sLog } from '../types/k8s';

export interface LogEntry {
  timestamp?: string;
  message: string;
  level?: string;
  pod?: string;
  container?: string;
  [key: string]: any;
}

/**
 * Filter logs by search query (supports regex)
 */
export function filterLogsBySearch(logs: string[], searchQuery: string): string[] {
  if (!searchQuery.trim()) {
    return logs;
  }
  
  try {
    // Try regex first
    const regex = new RegExp(searchQuery, 'i');
    return logs.filter(log => regex.test(log));
  } catch {
    // If regex fails, fall back to simple string search
    const query = searchQuery.toLowerCase();
    return logs.filter(log => log.toLowerCase().includes(query));
  }
}

/**
 * Filter logs by container name
 */
export function filterLogsByContainer(
  logs: string[],
  container: string,
  logEntries?: LogEntry[]
): string[] {
  if (!container || !logEntries) {
    return logs;
  }
  
  // If we have structured log entries, filter by container
  // Otherwise, return all logs (container filtering would need backend support)
  return logs;
}

/**
 * Apply multiple filters to logs
 */
export function applyLogFilters(
  logs: string[],
  filters: {
    searchQuery?: string;
    container?: string;
    tailLines?: number;
  }
): string[] {
  let filtered = [...logs];
  
  // Apply search filter
  if (filters.searchQuery) {
    filtered = filterLogsBySearch(filtered, filters.searchQuery);
  }
  
  // Tail lines is applied by backend, but we can limit here too if needed
  if (filters.tailLines && filtered.length > filters.tailLines) {
    filtered = filtered.slice(-filters.tailLines);
  }
  
  return filtered;
}

/**
 * Highlight search matches in log text
 */
export function highlightSearchMatches(text: string, searchQuery: string): string {
  if (!searchQuery.trim()) {
    return text;
  }
  
  try {
    const regex = new RegExp(`(${escapeRegex(searchQuery)})`, 'gi');
    return text.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>');
  } catch {
    // Fall back to simple string replacement
    const query = escapeHtml(searchQuery);
    const escapedText = escapeHtml(text);
    return escapedText.replace(
      new RegExp(`(${query})`, 'gi'),
      '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>'
    );
  }
}

/**
 * Extract structured log fields from JSON logs
 */
export function parseStructuredLog(log: string): {
  isStructured: boolean;
  data: Record<string, any> | null;
  rawMessage: string;
} {
  try {
    const parsed = JSON.parse(log);
    return {
      isStructured: true,
      data: parsed,
      rawMessage: log
    };
  } catch {
    return {
      isStructured: false,
      data: null,
      rawMessage: log
    };
  }
}

/**
 * Extract log level/severity from log message
 */
export function extractLogLevel(log: string): string {
  const upperLog = log.toUpperCase();
  
  if (upperLog.includes('ERROR') || upperLog.includes('ERR')) {
    return 'error';
  }
  if (upperLog.includes('WARN') || upperLog.includes('WARNING')) {
    return 'warn';
  }
  if (upperLog.includes('INFO')) {
    return 'info';
  }
  if (upperLog.includes('DEBUG')) {
    return 'debug';
  }
  
  // Try to parse from structured log
  try {
    const parsed = JSON.parse(log);
    return (parsed.level || parsed.severity || parsed.logLevel || 'info').toLowerCase();
  } catch {
    return 'info';
  }
}

/**
 * Filter logs by severity (hierarchical)
 */
export function filterLogsBySeverity(logs: K8sLog[], severity: string): K8sLog[] {
  if (!severity) return logs;
  
  const severityLevels: Record<string, number> = {
    'DEBUG': 0,
    'INFO': 1,
    'WARN': 2,
    'WARNING': 2,
    'ERROR': 3,
    'FATAL': 4,
    'CRITICAL': 4
  };
  
  const filterLevel = severityLevels[severity.toUpperCase()] ?? 0;
  
  return logs.filter(log => {
    const logLevel = severityLevels[log.level.toUpperCase()] ?? 0;
    return logLevel >= filterLevel;
  });
}

/**
 * Get log color based on level
 */
export function getLogLevelColor(level: string): string {
  const levelLower = level.toLowerCase();
  switch (levelLower) {
    case 'error':
    case 'err':
      return 'text-red-600 dark:text-red-400';
    case 'warn':
    case 'warning':
      return 'text-yellow-600 dark:text-yellow-400';
    case 'info':
      return 'text-blue-600 dark:text-blue-400';
    case 'debug':
      return 'text-gray-600 dark:text-gray-400';
    default:
      return 'text-foreground';
  }
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function escapeHtml(str: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return str.replace(/[&<>"']/g, m => map[m]);
}

