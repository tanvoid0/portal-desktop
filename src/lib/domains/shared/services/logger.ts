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

export interface LogEntry {
	timestamp: Date;
	level: LogLevel;
	message: string;
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
		enableConsole: true,
		enableFile: false,
		maxEntries: 1000
	};

	private entries: LogEntry[] = [];
	private listeners: ((entry: LogEntry) => void)[] = [];

	/**
	 * Configure the logger
	 */
	configure(config: Partial<LoggerConfig>): void {
		this.config = { ...this.config, ...config };
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
	 * Internal method to add log entry
	 */
	private addEntry(level: LogLevel, message: string, data?: any): void {
		if (level < this.config.level) return;

		const entry: LogEntry = {
			timestamp: new Date(),
			level,
			message,
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
		const context = entry.data?.context ? `[${entry.data.context}]` : '';
		const prefix = `[${timestamp}] [${levelName}]${context}`;

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
			debug: (message: string, data?: any) => this.debug(message, data ? { ...data, context } : { context }),
			info: (message: string, data?: any) => this.info(message, data ? { ...data, context } : { context }),
			warn: (message: string, data?: any) => this.warn(message, data ? { ...data, context } : { context }),
			error: (message: string, data?: any) => this.error(message, data ? { ...data, context } : { context })
		};
	}
}

// Export singleton instance
export const logger = new LoggerService();

// Export convenience functions
export const createLogger = (context: string) => logger.createScoped(context);
 