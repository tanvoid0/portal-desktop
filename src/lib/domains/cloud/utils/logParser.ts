// Log parsing utilities to convert raw log strings to structured K8sLog[]
import type { K8sLog } from '../types/k8s';

/**
 * Parse raw log string into structured log entries
 * Handles various log formats:
 * - Plain text logs
 * - Kubernetes log format (timestamp level message)
 * - JSON structured logs
 */
export function parseRawLogs(
  rawLogs: string,
  podName: string,
  containerName: string = 'app'
): K8sLog[] {
  if (!rawLogs || !rawLogs.trim()) {
    return [];
  }

  const lines = rawLogs.split('\n').filter(line => line.trim());
  const logs: K8sLog[] = [];

  for (const line of lines) {
    // Try to parse as JSON first
    let parsed: K8sLog | null = null;
    
    try {
      const jsonData = JSON.parse(line);
      // If it's a JSON log, extract fields
      parsed = {
        timestamp: jsonData.timestamp || jsonData.time || jsonData.ts || new Date().toISOString(),
        level: jsonData.level || jsonData.severity || jsonData.logLevel || 'INFO',
        message: jsonData.message || jsonData.msg || jsonData.text || line,
        pod: podName,
        container: jsonData.container || containerName
      };
    } catch {
      // Not JSON, try to parse as Kubernetes log format
      // Common formats:
      // - "2024-01-01T12:00:00Z INFO message"
      // - "2024-01-01 12:00:00 [INFO] message"
      // - "INFO message"
      
      const timestampMatch = line.match(/^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)/);
      const levelMatch = line.match(/\[?([A-Z]+)\]?/);
      
      if (timestampMatch && levelMatch) {
        const timestamp = timestampMatch[1];
        const level = levelMatch[1];
        const messageStart = timestampMatch[0].length + levelMatch[0].length + 1;
        const message = line.substring(messageStart).trim();
        
        parsed = {
          timestamp: timestamp,
          level: level,
          message: message || line,
          pod: podName,
          container: containerName
        };
      } else if (levelMatch) {
        // Just level, no timestamp
        const level = levelMatch[1];
        const messageStart = levelMatch[0].length + 1;
        const message = line.substring(messageStart).trim();
        
        parsed = {
          timestamp: new Date().toISOString(),
          level: level,
          message: message || line,
          pod: podName,
          container: containerName
        };
      } else {
        // Plain text log
        parsed = {
          timestamp: new Date().toISOString(),
          level: extractLogLevel(line),
          message: line,
          pod: podName,
          container: containerName
        };
      }
    }

    if (parsed) {
      logs.push(parsed);
    }
  }

  return logs;
}

/**
 * Extract log level from a message string
 */
function extractLogLevel(message: string): string {
  const upperMessage = message.toUpperCase();
  
  if (upperMessage.includes('ERROR') || upperMessage.includes('FATAL') || upperMessage.includes('CRITICAL')) {
    return 'ERROR';
  }
  if (upperMessage.includes('WARN') || upperMessage.includes('WARNING')) {
    return 'WARN';
  }
  if (upperMessage.includes('INFO') || upperMessage.includes('INFORMATION')) {
    return 'INFO';
  }
  if (upperMessage.includes('DEBUG') || upperMessage.includes('TRACE')) {
    return 'DEBUG';
  }
  
  return 'INFO'; // Default
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

