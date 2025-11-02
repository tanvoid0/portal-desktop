/**
 * Production Performance Monitoring Utilities
 * Tracks component performance and provides optimization insights
 */

interface PerformanceMetric {
	name: string;
	startTime: number;
	endTime?: number;
	duration?: number;
	metadata?: Record<string, unknown>;
}

class PerformanceMonitor {
	private metrics: Map<string, PerformanceMetric> = new Map();
	private observers: PerformanceObserver[] = [];

	constructor() {
		this.setupPerformanceObserver();
	}

	/**
	 * Start timing a performance metric
	 */
	start(name: string, metadata?: Record<string, any>): void {
		this.metrics.set(name, {
			name,
			startTime: performance.now(),
			metadata
		});
	}

	/**
	 * End timing a performance metric
	 */
	end(name: string): number | null {
		const metric = this.metrics.get(name);
		if (!metric) {
			console.warn(`Performance metric "${name}" not found`);
			return null;
		}

		metric.endTime = performance.now();
		metric.duration = metric.endTime - metric.startTime;

		// Log slow operations
		if (metric.duration > 100) {
			console.warn(`Slow operation detected: ${name} took ${metric.duration.toFixed(2)}ms`);
		}

		return metric.duration;
	}

	/**
	 * Get performance metrics
	 */
	getMetrics(): PerformanceMetric[] {
		return Array.from(this.metrics.values());
	}

	/**
	 * Clear all metrics
	 */
	clear(): void {
		this.metrics.clear();
	}

	/**
	 * Setup performance observer for automatic monitoring
	 */
	private setupPerformanceObserver(): void {
		if (typeof window === 'undefined') return;

		// Monitor long tasks
		if ('PerformanceObserver' in window) {
			try {
				const longTaskObserver = new PerformanceObserver((list) => {
					for (const entry of list.getEntries()) {
						if (entry.duration > 50) {
							console.warn(`Long task detected: ${entry.duration.toFixed(2)}ms`);
						}
					}
				});
				longTaskObserver.observe({ entryTypes: ['longtask'] });
				this.observers.push(longTaskObserver);
			} catch (e) {
				// Long task observer not supported
			}

			// Monitor layout shifts
			try {
				const layoutShiftObserver = new PerformanceObserver((list) => {
					for (const entry of list.getEntries()) {
						// Check if entry has hadRecentInput property (LayoutShiftEntry)
						if ('hadRecentInput' in entry && (entry as { hadRecentInput: boolean }).hadRecentInput) continue;
						
						// Check if entry has value property (LayoutShiftEntry)
						if ('value' in entry) {
							const layoutShift = entry as { value: number };
							if (layoutShift.value > 0.1) {
								console.warn(`Layout shift detected: ${layoutShift.value.toFixed(3)}`);
							}
						}
					}
				});
				layoutShiftObserver.observe({ entryTypes: ['layout-shift'] });
				this.observers.push(layoutShiftObserver);
			} catch (e) {
				// Layout shift observer not supported
			}
		}
	}

	/**
	 * Cleanup observers
	 */
	destroy(): void {
		this.observers.forEach(observer => observer.disconnect());
		this.observers = [];
	}
}

// Global performance monitor instance
export const performanceMonitor = new PerformanceMonitor();

/**
 * Performance decorator for functions
 */
export function measurePerformance<T extends (...args: unknown[]) => unknown>(
	fn: T,
	name?: string
): T {
	return ((...args: Parameters<T>) => {
		const metricName = name || fn.name || 'anonymous';
		performanceMonitor.start(metricName);
		
		try {
			const result = fn(...args);
			
			// Handle async functions
			if (result instanceof Promise) {
				return result.finally(() => {
					performanceMonitor.end(metricName);
				});
			}
			
			performanceMonitor.end(metricName);
			return result;
		} catch (error) {
			performanceMonitor.end(metricName);
			throw error;
		}
	}) as T;
}

/**
 * Component performance tracking
 */
export function trackComponentPerformance(componentName: string) {
	return {
		start: () => performanceMonitor.start(`component-${componentName}`),
		end: () => performanceMonitor.end(`component-${componentName}`)
	};
}

/**
 * Memory usage monitoring
 */
export function getMemoryUsage(): {
	used: number;
	total: number;
	percentage: number;
} | null {
	if (typeof window === 'undefined' || !('memory' in performance)) {
		return null;
	}

	const memory = (performance as any).memory;
	return {
		used: memory.usedJSHeapSize,
		total: memory.totalJSHeapSize,
		percentage: (memory.usedJSHeapSize / memory.totalJSHeapSize) * 100
	};
}

/**
 * Network performance monitoring
 */
export function getNetworkInfo(): {
	effectiveType: string;
	downlink: number;
	rtt: number;
} | null {
	if (typeof window === 'undefined' || !('connection' in navigator)) {
		return null;
	}

	const connection = (navigator as any).connection;
	return {
		effectiveType: connection.effectiveType,
		downlink: connection.downlink,
		rtt: connection.rtt
	};
}

/**
 * Bundle size monitoring
 */
export function getBundleSize(): {
	scripts: number;
	styles: number;
	total: number;
} {
	if (typeof window === 'undefined') {
		return { scripts: 0, styles: 0, total: 0 };
	}

	const scripts = document.querySelectorAll('script[src]');
	const styles = document.querySelectorAll('link[rel="stylesheet"]');
	
	let scriptSize = 0;
	let styleSize = 0;

	scripts.forEach(script => {
		const src = script.getAttribute('src');
		if (src && !src.startsWith('data:')) {
			// Estimate size based on URL length (rough approximation)
			scriptSize += src.length * 10;
		}
	});

	styles.forEach(style => {
		const href = style.getAttribute('href');
		if (href && !href.startsWith('data:')) {
			styleSize += href.length * 10;
		}
	});

	return {
		scripts: scriptSize,
		styles: styleSize,
		total: scriptSize + styleSize
	};
}

/**
 * Performance report generator
 */
export function generatePerformanceReport(): {
	metrics: PerformanceMetric[];
	memory: ReturnType<typeof getMemoryUsage>;
	network: ReturnType<typeof getNetworkInfo>;
	bundle: ReturnType<typeof getBundleSize>;
	timestamp: number;
} {
	return {
		metrics: performanceMonitor.getMetrics(),
		memory: getMemoryUsage(),
		network: getNetworkInfo(),
		bundle: getBundleSize(),
		timestamp: Date.now()
	};
}
