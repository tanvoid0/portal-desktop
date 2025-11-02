/**
 * Event bus service for Portal Desktop
 * Provides pub/sub pattern for inter-component communication
 */

export interface EventListener<T = any> {
	(event: T): void;
}

export interface EventSubscription {
	unsubscribe: () => void;
}

class EventBusService {
	private listeners = new Map<string, Set<EventListener>>();

	/**
	 * Subscribe to an event
	 */
	on<T = any>(event: string, listener: EventListener<T>): EventSubscription {
		if (!this.listeners.has(event)) {
			this.listeners.set(event, new Set());
		}

		const eventListeners = this.listeners.get(event)!;
		eventListeners.add(listener);

		return {
			unsubscribe: () => {
				eventListeners.delete(listener);
				if (eventListeners.size === 0) {
					this.listeners.delete(event);
				}
			}
		};
	}

	/**
	 * Subscribe to an event once (auto-unsubscribe after first emission)
	 */
	once<T = any>(event: string, listener: EventListener<T>): EventSubscription {
		const subscription = this.on(event, (data: T) => {
			listener(data);
			subscription.unsubscribe();
		});

		return subscription;
	}

	/**
	 * Emit an event to all subscribers
	 */
	emit<T = any>(event: string, data?: T): void {
		const eventListeners = this.listeners.get(event);
		if (!eventListeners) return;

		// Create a copy to avoid issues if listeners modify the set during iteration
		const listeners = Array.from(eventListeners);
		listeners.forEach(listener => {
			try {
				listener(data);
			} catch (error) {
				console.error(`Error in event listener for "${event}":`, error);
			}
		});
	}

	/**
	 * Remove all listeners for an event
	 */
	off(event: string): void {
		this.listeners.delete(event);
	}

	/**
	 * Remove a specific listener for an event
	 */
	removeListener<T = any>(event: string, listener: EventListener<T>): void {
		const eventListeners = this.listeners.get(event);
		if (eventListeners) {
			eventListeners.delete(listener);
			if (eventListeners.size === 0) {
				this.listeners.delete(event);
			}
		}
	}

	/**
	 * Get all registered events
	 */
	getEvents(): string[] {
		return Array.from(this.listeners.keys());
	}

	/**
	 * Get listener count for an event
	 */
	getListenerCount(event: string): number {
		return this.listeners.get(event)?.size || 0;
	}

	/**
	 * Clear all listeners
	 */
	clear(): void {
		this.listeners.clear();
	}

	/**
	 * Create a scoped event bus for a specific namespace
	 */
	createScoped(namespace: string) {
		return {
			on: <T = any>(event: string, listener: EventListener<T>) => 
				this.on(`${namespace}:${event}`, listener),
			once: <T = any>(event: string, listener: EventListener<T>) => 
				this.once(`${namespace}:${event}`, listener),
			emit: <T = any>(event: string, data?: T) => 
				this.emit(`${namespace}:${event}`, data),
			off: (event: string) => this.off(`${namespace}:${event}`),
			removeListener: <T = any>(event: string, listener: EventListener<T>) => 
				this.removeListener(`${namespace}:${event}`, listener)
		};
	}
}

// Export singleton instance
export const eventBus = new EventBusService();

// Export convenience functions
export const createEventBus = (namespace: string) => eventBus.createScoped(namespace);
