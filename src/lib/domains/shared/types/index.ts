/**
 * Shared Domain Types
 * 
 * Common types and interfaces used across all domains
 */

// Logger Types
export interface LogContext {
  context: string;
  data?: Record<string, any>;
  error?: Error;
  traceId?: string;
}

export interface LogEntry {
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  context: LogContext;
  timestamp: Date;
}

// Cache Types
export interface CacheEntry<T = any> {
  data: T;
  timestamp: number;
  ttl: number;
}

export interface CacheStats {
  hits: number;
  misses: number;
  size: number;
  maxSize: number;
}

// Event Bus Types
export interface DomainEvent {
  type: string;
  payload: any;
  timestamp: Date;
  source: string;
}

export interface EventSubscription {
  id: string;
  eventType: string;
  callback: (event: DomainEvent) => void;
}

// System Types
export interface SystemInfo {
  platform: string;
  arch: string;
  version: string;
  nodeVersion: string;
  memory: {
    total: number;
    free: number;
    used: number;
  };
}

export interface DockerStatus {
  isRunning: boolean;
  version?: string;
  containers: number;
  images: number;
}

// Error Types
export interface DomainError extends Error {
  code: string;
  context: string;
  data?: Record<string, any>;
}

// API Types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginationParams {
  page: number;
  limit: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

// Process Types
export interface ProcessInfo {
  id: string;
  name: string;
  status: 'running' | 'stopped' | 'error';
  pid?: number;
  startTime?: Date;
  endTime?: Date;
  logs: string[];
}

// Master Password Types
export interface MasterPasswordInfo {
  isSet: boolean;
  lastChanged?: Date;
  strength: 'weak' | 'medium' | 'strong';
}

// Generic Types
export type LoadingState = 'idle' | 'loading' | 'success' | 'error';

export interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  lastUpdated?: Date;
}
