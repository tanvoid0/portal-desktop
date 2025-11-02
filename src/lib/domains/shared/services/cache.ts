/**
 * In-memory cache service for Portal Desktop
 * Provides TTL-based caching with automatic cleanup
 */

export interface CacheEntry<T = any> {
	value: T;
	expiresAt: number;
	createdAt: number;
}

export interface CacheConfig {
	defaultTTL: number; // in milliseconds
	maxSize: number;
	cleanupInterval: number; // in milliseconds
}

class CacheService {
	private cache = new Map<string, CacheEntry>();
	private config: CacheConfig = {
		defaultTTL: 5 * 60 * 1000, // 5 minutes
		maxSize: 1000,
		cleanupInterval: 60 * 1000 // 1 minute
	};
	private cleanupTimer?: NodeJS.Timeout;

	constructor() {
		this.startCleanup();
	}

	/**
	 * Configure cache settings
	 */
	configure(config: Partial<CacheConfig>): void {
		this.config = { ...this.config, ...config };
	}

	/**
	 * Set a value in the cache
	 */
	set<T>(key: string, value: T, ttl?: number): void {
		const now = Date.now();
		const expiresAt = now + (ttl || this.config.defaultTTL);

		// Remove oldest entries if cache is full
		if (this.cache.size >= this.config.maxSize) {
			this.evictOldest();
		}

		this.cache.set(key, {
			value,
			expiresAt,
			createdAt: now
		});
	}

	/**
	 * Get a value from the cache
	 */
	get<T>(key: string): T | null {
		const entry = this.cache.get(key);
		
		if (!entry) {
			return null;
		}

		// Check if expired
		if (Date.now() > entry.expiresAt) {
			this.cache.delete(key);
			return null;
		}

		return entry.value as T;
	}

	/**
	 * Check if a key exists and is not expired
	 */
	has(key: string): boolean {
		const entry = this.cache.get(key);
		
		if (!entry) {
			return false;
		}

		// Check if expired
		if (Date.now() > entry.expiresAt) {
			this.cache.delete(key);
			return false;
		}

		return true;
	}

	/**
	 * Delete a key from the cache
	 */
	delete(key: string): boolean {
		return this.cache.delete(key);
	}

	/**
	 * Clear all cache entries
	 */
	clear(): void {
		this.cache.clear();
	}

	/**
	 * Get cache statistics
	 */
	getStats() {
		const now = Date.now();
		let expired = 0;
		let active = 0;

		for (const entry of this.cache.values()) {
			if (now > entry.expiresAt) {
				expired++;
			} else {
				active++;
			}
		}

		return {
			total: this.cache.size,
			active,
			expired,
			maxSize: this.config.maxSize
		};
	}

	/**
	 * Get all active keys
	 */
	getKeys(): string[] {
		const now = Date.now();
		const keys: string[] = [];

		for (const [key, entry] of this.cache.entries()) {
			if (now <= entry.expiresAt) {
				keys.push(key);
			}
		}

		return keys;
	}

	/**
	 * Remove the oldest entry
	 */
	private evictOldest(): void {
		let oldestKey = '';
		let oldestTime = Infinity;

		for (const [key, entry] of this.cache.entries()) {
			if (entry.createdAt < oldestTime) {
				oldestTime = entry.createdAt;
				oldestKey = key;
			}
		}

		if (oldestKey) {
			this.cache.delete(oldestKey);
		}
	}

	/**
	 * Start automatic cleanup of expired entries
	 */
	private startCleanup(): void {
		this.cleanupTimer = setInterval(() => {
			this.cleanup();
		}, this.config.cleanupInterval);
	}

	/**
	 * Clean up expired entries
	 */
	private cleanup(): void {
		const now = Date.now();
		const expiredKeys: string[] = [];

		for (const [key, entry] of this.cache.entries()) {
			if (now > entry.expiresAt) {
				expiredKeys.push(key);
			}
		}

		expiredKeys.forEach(key => this.cache.delete(key));
	}

	/**
	 * Stop cleanup timer
	 */
	destroy(): void {
		if (this.cleanupTimer) {
			clearInterval(this.cleanupTimer);
			this.cleanupTimer = undefined;
		}
	}
}

// Export singleton instance
export const cache = new CacheService();
