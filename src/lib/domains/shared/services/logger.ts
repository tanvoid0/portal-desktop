/**
 * Centralized logging service for Portal Desktop
 * Provides structured logging with different levels and contexts
 */

export enum LogLevel {
	DEBUG = 0,
	INFO = 1,
	WARN = 2,
	ERROR = 3
}

export type LogSource = 'frontend' | 'backend';

export interface LogEntry {
	timestamp: Date;
	level: LogLevel;
	message: string;
	source: LogSource;
	context?: string;
	data?: any;
}

export interface LoggerConfig {
	level: LogLevel;
	enableConsole: boolean;
	enableFile: boolean;
	maxEntries: number;
}

class LoggerService {
	private config: LoggerConfig = {
		level: LogLevel.INFO,
		enableConsole: import.meta.env.DEV, // Only enable console in development
		enableFile: false,
		maxEntries: 1000
	};

	private entries: LogEntry[] = [];
	private listeners: ((entry: LogEntry) => void)[] = [];
	private backendLogListener: (() => void) | null = null;

	/**
	 * Configure the logger
	 */
	configure(config: Partial<LoggerConfig>): void {
		this.config = { ...this.config, ...config };
		// Ensure console is disabled in production unless explicitly enabled
		if (!import.meta.env.DEV && config.enableConsole !== true) {
			this.config.enableConsole = false;
		}
	}

	/**
	 * Add a log entry listener
	 */
	onLog(listener: (entry: LogEntry) => void): () => void {
		this.listeners.push(listener);
		return () => {
			const index = this.listeners.indexOf(listener);
			if (index > -1) {
				this.listeners.splice(index, 1);
			}
		};
	}

	/**
	 * Get all log entries
	 */
	getEntries(): LogEntry[] {
		return [...this.entries];
	}

	/**
	 * Clear all log entries
	 */
	clear(): void {
		this.entries = [];
	}

	/**
	 * Initialize backend log listener (Tauri events)
	 */
	initBackendLogListener(): void {
		// Only initialize if in Tauri environment
		if (typeof window === 'undefined' || !('__TAURI__' in window)) {
			return;
		}

		// Clean up existing listener
		if (this.backendLogListener) {
			this.backendLogListener();
		}

		import('@tauri-apps/api/event').then(({ listen }) => {
			listen<{ level: string; context?: string; message: string }>('backend-log', (event) => {
				const { level, context, message } = event.payload;
				
				// Map backend log level to frontend LogLevel
				let logLevel: LogLevel;
				switch (level.toUpperCase()) {
					case 'DEBUG':
						logLevel = LogLevel.DEBUG;
						break;
					case 'INFO':
						logLevel = LogLevel.INFO;
						break;
					case 'WARN':
						logLevel = LogLevel.WARN;
						break;
					case 'ERROR':
						logLevel = LogLevel.ERROR;
						break;
					default:
						logLevel = LogLevel.INFO;
				}

				this.addEntry(logLevel, message, { context }, 'backend', context);
			}).then((unlisten) => {
				this.backendLogListener = unlisten;
			}).catch((err) => {
				console.error('Failed to set up backend log listener:', err);
			});
		});
	}

	/**
	 * Internal method to add log entry
	 */
	private addEntry(
		level: LogLevel, 
		message: string, 
		data?: any, 
		source: LogSource = 'frontend',
		context?: string
	): void {
		if (level < this.config.level) return;

		const entry: LogEntry = {
			timestamp: new Date(),
			level,
			message,
			source,
			context,
			data
		};

		// Add to entries array
		this.entries.push(entry);

		// Maintain max entries limit
		if (this.entries.length > this.config.maxEntries) {
			this.entries = this.entries.slice(-this.config.maxEntries);
		}

		// Console output
		if (this.config.enableConsole) {
			this.logToConsole(entry);
		}

		// Notify listeners
		this.listeners.forEach(listener => listener(entry));
	}

	/**
	 * Log to console with appropriate styling
	 */
	private logToConsole(entry: LogEntry): void {
		const timestamp = entry.timestamp.toISOString();
		const levelName = LogLevel[entry.level];
		const sourceTag = `[${entry.source.toUpperCase()}]`;
		const contextTag = entry.context ? `[${entry.context}]` : '';
		const prefix = `[${timestamp}] ${sourceTag} [${levelName}]${contextTag}`;

		// Format data for better console output
		let dataStr = '';
		if (entry.data) {
			if (typeof entry.data === 'string') {
				dataStr = entry.data;
			} else {
				dataStr = JSON.stringify(entry.data, null, 2);
			}
		}
		const fullMessage = dataStr ? `${entry.message} ${dataStr}` : entry.message;

		switch (entry.level) {
			case LogLevel.DEBUG:
				console.debug(prefix, fullMessage);
				break;
			case LogLevel.INFO:
				console.info(prefix, fullMessage);
				break;
			case LogLevel.WARN:
				console.warn(prefix, fullMessage);
				break;
			case LogLevel.ERROR:
				console.error(prefix, fullMessage);
				break;
		}
	}

	/**
	 * Debug level logging
	 */
	debug(message: string, data?: any): void {
		this.addEntry(LogLevel.DEBUG, message, data);
	}

	/**
	 * Info level logging
	 */
	info(message: string, data?: any): void {
		this.addEntry(LogLevel.INFO, message, data);
	}

	/**
	 * Warning level logging
	 */
	warn(message: string, data?: any): void {
		this.addEntry(LogLevel.WARN, message, data);
	}

	/**
	 * Error level logging
	 */
	error(message: string, data?: any): void {
		this.addEntry(LogLevel.ERROR, message, data);
	}

	/**
	 * Create a scoped logger for a specific context
	 */
	createScoped(context: string) {
		return {
			debug: (message: string, data?: any) => {
				this.addEntry(LogLevel.DEBUG, message, data ? { ...data, context } : { context }, 'frontend', context);
			},
			info: (message: string, data?: any) => {
				this.addEntry(LogLevel.INFO, message, data ? { ...data, context } : { context }, 'frontend', context);
			},
			warn: (message: string, data?: any) => {
				this.addEntry(LogLevel.WARN, message, data ? { ...data, context } : { context }, 'frontend', context);
			},
			error: (message: string, data?: any) => {
				this.addEntry(LogLevel.ERROR, message, data ? { ...data, context } : { context }, 'frontend', context);
			}
		};
	}

	/**
	 * Get filtered entries
	 */
	getFilteredEntries(filters?: {
		level?: LogLevel;
		source?: LogSource;
		context?: string;
		search?: string;
	}): LogEntry[] {
		let filtered = [...this.entries];

		if (filters?.level !== undefined) {
			filtered = filtered.filter(entry => entry.level === filters.level);
		}

		if (filters?.source) {
			filtered = filtered.filter(entry => entry.source === filters.source);
		}

		if (filters?.context) {
			filtered = filtered.filter(entry => entry.context === filters.context);
		}

		if (filters?.search) {
			const searchLower = filters.search.toLowerCase();
			filtered = filtered.filter(entry => 
				entry.message.toLowerCase().includes(searchLower) ||
				entry.context?.toLowerCase().includes(searchLower)
			);
		}

		return filtered;
	}
}

// Export singleton instance
export const logger = new LoggerService();

// Export convenience functions
export const createLogger = (context: string) => logger.createScoped(context);
 