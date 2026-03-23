import { writable } from 'svelte/store';
import type { DashboardOverview } from '../services/dashboardService';
import {
	DASHBOARD_OVERVIEW_CACHE_KEY,
	getDashboardOverview,
	invalidateDashboardOverview,
	primeDashboardOverview
} from '../services/dashboardService';

interface DashboardState {
	overview: DashboardOverview | null;
	loading: boolean;
	error: string | null;
}

const initialState: DashboardState = {
	overview: null,
	loading: false,
	error: null
};

function createDashboardStore() {
	const { subscribe, set, update } = writable<DashboardState>(initialState);

	let refreshPromise: Promise<void> | null = null;

	async function refresh(force: boolean = true): Promise<void> {
		if (refreshPromise) return refreshPromise;

		refreshPromise = (async () => {
			update(s => ({ ...s, loading: true, error: null }));

			const overview = await getDashboardOverview({ force });

			// Cache for TTL so badges stay populated across navigation.
			primeDashboardOverview(overview);

			set({
				overview,
				loading: false,
				error: null
			});
		})().catch((e: unknown) => {
			const message = e instanceof Error ? e.message : String(e);
			update(s => ({ ...s, loading: false, error: message }));
		}).finally(() => {
			refreshPromise = null;
		});

		return refreshPromise;
	}

	async function load(): Promise<void> {
		// TTL-aware service call; avoids extra network request when cached.
		return refresh(false);
	}

	function invalidate(): void {
		// Mark stale by clearing the cache entry.
		// Keep existing `overview` so UI doesn't briefly lose badges.
		invalidateDashboardOverview();
	}

	function clear(): void {
		invalidateDashboardOverview();
		set(initialState);
	}

	return {
		subscribe,
		load,
		refresh,
		invalidate,
		clear,
		// Exposed for debugging/guards; not required for UI.
		_cacheKey: DASHBOARD_OVERVIEW_CACHE_KEY
	};
}

export const dashboardStore = createDashboardStore();

