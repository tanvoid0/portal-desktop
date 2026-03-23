/**
 * Gradient utility functions for consistent gradient usage across the app
 * All gradients are designed to work in both light and dark modes
 */

export const gradientClasses = {
	// Subtle background gradients
	subtle: 'bg-gradient-subtle',
	card: 'bg-gradient-card',
	surface: 'bg-gradient-surface',
	
	// Status-based gradients
	success: 'bg-gradient-to-br from-success-50 to-success-100 dark:from-success-950/20 dark:to-success-900/10',
	warning: 'bg-gradient-to-br from-warning-50 to-warning-100 dark:from-warning-950/20 dark:to-warning-900/10',
	error: 'bg-gradient-to-br from-error-50 to-error-100 dark:from-error-950/20 dark:to-error-900/10',
	info: 'bg-gradient-to-br from-info-50 to-info-100 dark:from-info-950/20 dark:to-info-900/10',
	
	// Primary gradients
	primary: 'bg-gradient-to-br from-primary-50 to-primary-100 dark:from-primary-950/20 dark:to-primary-900/10',
	primaryStrong: 'bg-gradient-to-br from-primary-500 to-primary-600 dark:from-primary-600 dark:to-primary-700',
	
	// Text gradients
	textPrimary: 'bg-gradient-to-r from-primary-600 to-primary-400 bg-clip-text text-transparent dark:from-primary-400 dark:to-primary-300',
	textSuccess: 'bg-gradient-to-r from-success-600 to-success-400 bg-clip-text text-transparent dark:from-success-400 dark:to-success-300',
	textWarning: 'bg-gradient-to-r from-warning-600 to-warning-400 bg-clip-text text-transparent dark:from-warning-400 dark:to-warning-300',
	textError: 'bg-gradient-to-r from-error-600 to-error-400 bg-clip-text text-transparent dark:from-error-400 dark:to-error-300',
	
	// Overlay gradients
	overlayTop: 'bg-gradient-to-b from-black/5 to-transparent dark:from-white/5',
	overlayBottom: 'bg-gradient-to-t from-black/5 to-transparent dark:from-white/5',
	overlayLeft: 'bg-gradient-to-r from-black/5 to-transparent dark:from-white/5',
	overlayRight: 'bg-gradient-to-l from-black/5 to-transparent dark:from-white/5',
	
	// Card-specific gradients
	cardElevated: 'bg-gradient-to-br from-card-elevated via-card to-card-surface',
	cardHover: 'bg-gradient-to-br from-primary/5 via-transparent to-transparent',
};

/**
 * Get gradient class for a specific status
 */
export function getStatusGradient(status: 'success' | 'warning' | 'error' | 'info'): string {
	return gradientClasses[status];
}

/**
 * Get gradient class for text with a specific color
 */
export function getTextGradient(color: 'primary' | 'success' | 'warning' | 'error'): string {
	return gradientClasses[`text${color.charAt(0).toUpperCase() + color.slice(1)}` as keyof typeof gradientClasses];
}

/**
 * Apply gradient overlay to an element
 */
export function getOverlayGradient(direction: 'top' | 'bottom' | 'left' | 'right'): string {
	return gradientClasses[`overlay${direction.charAt(0).toUpperCase() + direction.slice(1)}` as keyof typeof gradientClasses];
}

