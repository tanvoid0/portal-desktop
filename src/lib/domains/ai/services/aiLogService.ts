import { invoke } from '@tauri-apps/api/core';
import type { AILog, LogFilters } from '../types/index.js';

export class AILogService {
  /**
   * Get logs with filters
   */
  async getLogs(filters: LogFilters = {}): Promise<AILog[]> {
    return invoke<AILog[]>('ai_get_logs', { filters });
  }

  /**
   * Search logs by content
   */
  async searchLogs(query: string, filters: LogFilters = {}): Promise<AILog[]> {
    return invoke<AILog[]>('ai_search_logs', { query, filters });
  }

  /**
   * Export logs to file
   */
  async exportLogs(filters: LogFilters = {}): Promise<string> {
    return invoke<string>('ai_export_logs', { filters });
  }
}

export const aiLogService = new AILogService();

