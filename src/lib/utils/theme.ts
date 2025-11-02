/**
 * Theme utilities for consistent theming across components
 * Provides utilities for creating theme-aware component variants
 */

import { tv, type VariantProps } from 'tailwind-variants';

/**
 * Base component styles that all themed components should inherit
 */
export const baseComponentStyles = {
	// Focus styles
	focus: 'focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
	
	// Disabled styles
	disabled: 'disabled:pointer-events-none disabled:opacity-50',
	
	// Transition styles
	transition: 'transition-all duration-200',
	
	// Border radius
	rounded: 'rounded-md',
	
	// Shadow styles
	shadow: 'shadow-sm',
	shadowHover: 'hover:shadow-md',
	
	// Ring styles for focus
	ring: 'focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
	
	// Animation styles
	animate: 'animate-in fade-in-0 zoom-in-95'
};

/**
 * Color variants for components
 */
export const colorVariants = {
	default: 'bg-primary text-primary-foreground hover:bg-primary/90',
	primary: 'bg-primary text-primary-foreground hover:bg-primary/90',
	secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
	destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
	outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
	ghost: 'hover:bg-accent hover:text-accent-foreground',
	link: 'text-primary underline-offset-4 hover:underline'
};

/**
 * Size variants for components
 */
export const sizeVariants = {
	sm: 'h-8 px-3 text-sm',
	default: 'h-9 px-4 py-2',
	lg: 'h-10 px-6',
	icon: 'size-9'
};

/**
 * Create a themed component variant
 */
export function createThemedVariant(baseStyles: string, variants: Record<string, any>) {
	return tv({
		base: `${baseStyles} ${baseComponentStyles.transition}`,
		variants,
		compoundVariants: [
			{
				variant: 'default',
				class: `${colorVariants.default} ${baseComponentStyles.focus}`
			},
			{
				variant: 'primary',
				class: `${colorVariants.primary} ${baseComponentStyles.focus}`
			},
			{
				variant: 'secondary',
				class: `${colorVariants.secondary} ${baseComponentStyles.focus}`
			},
			{
				variant: 'destructive',
				class: `${colorVariants.destructive} ${baseComponentStyles.focus}`
			},
			{
				variant: 'outline',
				class: `${colorVariants.outline} ${baseComponentStyles.focus}`
			},
			{
				variant: 'ghost',
				class: `${colorVariants.ghost} ${baseComponentStyles.focus}`
			},
			{
				variant: 'link',
				class: `${colorVariants.link} ${baseComponentStyles.focus}`
			}
		]
	});
}

/**
 * Theme-aware color utilities
 */
export const themeColors = {
	// Background colors
	background: 'bg-background',
	card: 'bg-card',
	popover: 'bg-popover',
	sidebar: 'bg-sidebar',
	
	// Text colors
	foreground: 'text-foreground',
	muted: 'text-muted-foreground',
	accent: 'text-accent-foreground',
	
	// Border colors
	border: 'border-border',
	input: 'border-input',
	ring: 'ring-ring',
	
	// Interactive colors
	primary: 'bg-primary text-primary-foreground',
	secondary: 'bg-secondary text-secondary-foreground',
	destructive: 'bg-destructive text-destructive-foreground'
};

/**
 * Create responsive theme classes
 */
export function createResponsiveTheme(theme: 'light' | 'dark' | 'auto') {
	const baseClasses = 'transition-colors duration-200';
	
	switch (theme) {
		case 'light':
			return `${baseClasses} light`;
		case 'dark':
			return `${baseClasses} dark`;
		case 'auto':
		default:
			return baseClasses;
	}
}

/**
 * Theme context for components
 */
export interface ThemeContext {
	theme: 'light' | 'dark' | 'system';
	resolvedTheme: 'light' | 'dark';
	variant: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
	size: 'sm' | 'default' | 'lg' | 'icon';
}

/**
 * Get theme-aware classes for a component
 */
export function getThemeClasses(context: Partial<ThemeContext>): string {
	const classes = [baseComponentStyles.transition];
	
	if (context.variant) {
		classes.push(colorVariants[context.variant] || colorVariants.primary);
	}
	
	if (context.size) {
		classes.push(sizeVariants[context.size] || sizeVariants.default);
	}
	
	return classes.join(' ');
}

/**
 * CSS custom properties for dynamic theming
 */
export const cssVariables = {
	// Base colors
	background: '--background',
	foreground: '--foreground',
	card: '--card',
	cardForeground: '--card-foreground',
	popover: '--popover',
	popoverForeground: '--popover-foreground',
	
	// Interactive colors
	primary: '--primary',
	primaryForeground: '--primary-foreground',
	secondary: '--secondary',
	secondaryForeground: '--secondary-foreground',
	destructive: '--destructive',
	destructiveForeground: '--destructive-foreground',
	
	// Utility colors
	muted: '--muted',
	mutedForeground: '--muted-foreground',
	accent: '--accent',
	accentForeground: '--accent-foreground',
	
	// Border and input
	border: '--border',
	input: '--input',
	ring: '--ring',
	
	// Sidebar
	sidebar: '--sidebar',
	sidebarForeground: '--sidebar-foreground',
	sidebarPrimary: '--sidebar-primary',
	sidebarPrimaryForeground: '--sidebar-primary-foreground',
	sidebarAccent: '--sidebar-accent',
	sidebarAccentForeground: '--sidebar-accent-foreground',
	sidebarBorder: '--sidebar-border',
	sidebarRing: '--sidebar-ring'
};

/**
 * Apply theme to an element
 */
export function applyTheme(element: HTMLElement, theme: 'light' | 'dark' | 'system') {
	// Remove existing theme classes
	element.classList.remove('light', 'dark');
	
	// Add new theme class
	if (theme !== 'system') {
		element.classList.add(theme);
	}
	
	// Update CSS custom properties
	element.style.setProperty('--color-scheme', theme === 'system' ? 'auto' : theme);
}
