import { writable, derived } from 'svelte/store';
import { logger } from '../services/logger';

const log = logger.createScoped('ThemeStore');

export type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
	current: Theme;
	resolvedTheme: 'light' | 'dark';
}

// Create the theme store
function createThemeStore() {
	const { subscribe, set, update } = writable<ThemeState>({
		current: 'system',
		resolvedTheme: 'dark'
	});

	return {
		subscribe,
		
		// Initialize theme from localStorage
		initialize() {
			try {
				// Get saved theme preference from localStorage
				const savedTheme = localStorage.getItem('theme') as Theme || 'system';
				
				update(state => {
					const newState = {
						...state,
						current: savedTheme
					};
					
					// Calculate resolved theme based on current preference
					if (savedTheme === 'system') {
						// Use system preference via CSS media query
						newState.resolvedTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
					} else {
						newState.resolvedTheme = savedTheme;
					}
					
					return newState;
				});
				
				// Apply theme to DOM
				this.applyTheme();
				
				log.info('Theme initialized:', { current: savedTheme, resolved: this.getResolvedTheme() });
			} catch (error) {
				log.error('Failed to initialize theme:', error);
				// Fallback to system theme
				this.setTheme('system');
			}
		},
		
		// Set theme preference
		setTheme(theme: Theme) {
			update(state => {
				const newState = {
					...state,
					current: theme
				};
				
				// Calculate resolved theme
				if (theme === 'system') {
					newState.resolvedTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
				} else {
					newState.resolvedTheme = theme;
				}
				
				return newState;
			});
			
			// Save to localStorage
			localStorage.setItem('theme', theme);
			
			// Apply theme to DOM
			this.applyTheme();
			
			log.info('Theme changed to:', theme);
		},
		
		// Apply theme to DOM
		applyTheme() {
			const state = this.getCurrentState();
			const html = document.documentElement;
			
			// Remove existing theme classes
			html.classList.remove('light', 'dark');
			
			// For system theme, let Tailwind handle it via media queries
			// For explicit themes, add the class
			if (state.current !== 'system') {
				html.classList.add(state.resolvedTheme);
			}
			
			// Update meta theme-color for mobile browsers
			const metaThemeColor = document.querySelector('meta[name="theme-color"]');
			if (metaThemeColor) {
				metaThemeColor.setAttribute('content', 
					state.resolvedTheme === 'dark' ? '#0a0a0a' : '#ffffff'
				);
			}
			
			// Update CSS custom properties for better theming
			html.style.setProperty('--color-scheme', state.resolvedTheme);
			
			log.debug('Applied theme to DOM:', state.resolvedTheme);
		},
		
		// Get current state (for internal use)
		getCurrentState(): ThemeState {
			let currentState: ThemeState;
			subscribe(state => currentState = state)();
			return currentState!;
		},
		
		// Get resolved theme
		getResolvedTheme(): 'light' | 'dark' {
			return this.getCurrentState().resolvedTheme;
		},
		
		// Toggle between light and dark (ignores system)
		toggle() {
			const current = this.getCurrentState().current;
			if (current === 'system') {
				// If system, toggle to opposite of current resolved theme
				const newTheme = this.getCurrentState().resolvedTheme === 'light' ? 'dark' : 'light';
				this.setTheme(newTheme);
			} else {
				// If not system, toggle between light and dark
				const newTheme = current === 'light' ? 'dark' : 'light';
				this.setTheme(newTheme);
			}
		}
	};
}

// Create the store instance
export const themeStore = createThemeStore();

// Derived stores for convenience
export const currentTheme = derived(themeStore, $theme => $theme.current);
export const resolvedTheme = derived(themeStore, $theme => $theme.resolvedTheme);

// Listen for system theme changes (if supported)
if (typeof window !== 'undefined') {
	// Listen for system theme changes
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	
	const handleSystemThemeChange = (e: MediaQueryListEvent) => {
		// Only update if current theme is 'system'
		themeStore.subscribe(state => {
			if (state.current === 'system') {
				themeStore.setTheme('system'); // This will recalculate resolved theme
			}
		})();
	};
	
	// Add listener
	mediaQuery.addEventListener('change', handleSystemThemeChange);
	
	// Also listen for storage changes (in case theme is changed in another tab)
	window.addEventListener('storage', (e) => {
		if (e.key === 'theme') {
			const newTheme = e.newValue as Theme || 'system';
			themeStore.setTheme(newTheme);
		}
	});
}
