/**
 * Production-ready validation utilities for form components
 */

export interface ValidationRule {
	required?: boolean;
	minLength?: number;
	maxLength?: number;
	pattern?: RegExp;
	email?: boolean;
	url?: boolean;
	numeric?: boolean;
	positive?: boolean;
	custom?: (value: unknown) => string | null;
}

export interface ValidationResult {
	isValid: boolean;
	errors: string[];
	firstError: string | null;
}

/**
 * Validate a single value against rules
 */
export function validateValue(value: unknown, rules: ValidationRule): ValidationResult {
	const errors: string[] = [];

	// Required validation
	if (rules.required && (!value || (typeof value === 'string' && value.trim() === ''))) {
		errors.push('This field is required');
	}

	// Skip other validations if value is empty and not required
	if (!value && !rules.required) {
		return { isValid: true, errors: [], firstError: null };
	}

	// String length validations
	if (typeof value === 'string') {
		if (rules.minLength && value.length < rules.minLength) {
			errors.push(`Must be at least ${rules.minLength} characters`);
		}
		if (rules.maxLength && value.length > rules.maxLength) {
			errors.push(`Must be no more than ${rules.maxLength} characters`);
		}
	}

	// Pattern validation
	if (rules.pattern && typeof value === 'string' && !rules.pattern.test(value)) {
		errors.push('Invalid format');
	}

	// Email validation
	if (rules.email && typeof value === 'string' && !isValidEmail(value)) {
		errors.push('Must be a valid email address');
	}

	// URL validation
	if (rules.url && typeof value === 'string' && !isValidUrl(value)) {
		errors.push('Must be a valid URL');
	}

	// Numeric validation
	if (rules.numeric && !isNumeric(value)) {
		errors.push('Must be a number');
	}

	// Positive number validation
	if (rules.positive && (isNaN(Number(value)) || Number(value) <= 0)) {
		errors.push('Must be a positive number');
	}

	// Custom validation
	if (rules.custom) {
		const customError = rules.custom(value);
		if (customError) {
			errors.push(customError);
		}
	}

	return {
		isValid: errors.length === 0,
		errors,
		firstError: errors[0] || null
	};
}

/**
 * Validate multiple fields
 */
export function validateForm<T extends Record<string, unknown>>(
	data: T,
	rules: Record<keyof T, ValidationRule>
): Record<keyof T, ValidationResult> {
	const results = {} as Record<keyof T, ValidationResult>;

	for (const [field, rule] of Object.entries(rules)) {
		results[field as keyof T] = validateValue(data[field], rule);
	}

	return results;
}

/**
 * Check if form is valid
 */
export function isFormValid<T extends Record<string, any>>(
	validationResults: Record<keyof T, ValidationResult>
): boolean {
	return Object.values(validationResults).every(result => result.isValid);
}

/**
 * Get all form errors
 */
export function getFormErrors<T extends Record<string, any>>(
	validationResults: Record<keyof T, ValidationResult>
): Record<keyof T, string[]> {
	const errors = {} as Record<keyof T, string[]>;
	
	for (const [field, result] of Object.entries(validationResults)) {
		errors[field as keyof T] = result.errors;
	}
	
	return errors;
}

/**
 * Common validation rules
 */
export const commonRules = {
	required: { required: true },
	email: { email: true },
	url: { url: true },
	numeric: { numeric: true },
	positive: { positive: true },
	password: { 
		minLength: 8, 
		pattern: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/
	},
	phone: { 
		pattern: /^[\+]?[1-9][\d]{0,15}$/
	},
	username: {
		minLength: 3,
		maxLength: 20,
		pattern: /^[a-zA-Z0-9_]+$/
	}
};

/**
 * Helper functions
 */
function isValidEmail(email: string): boolean {
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	return emailRegex.test(email);
}

function isValidUrl(url: string): boolean {
	try {
		new URL(url);
		return true;
	} catch {
		return false;
	}
}

function isNumeric(value: unknown): boolean {
	return !isNaN(Number(value)) && isFinite(Number(value));
}

/**
 * Real-time validation hook for Svelte components
 */
export function createValidation<T extends Record<string, unknown>>(
	initialData: T,
	rules: Record<keyof T, ValidationRule>
) {
	let data = $state(initialData);
	let validationResults = $state({} as Record<keyof T, ValidationResult>);
	let isDirty = $state(false);

	// Validate all fields
	function validate() {
		validationResults = validateForm(data, rules);
		return validationResults;
	}

	// Validate single field
	function validateField(field: keyof T) {
		validationResults[field] = validateValue(data[field], rules[field]);
		return validationResults[field];
	}

	// Update field value and validate
	function updateField(field: keyof T, value: unknown) {
		(data as any)[field] = value;
		isDirty = true;
		validateField(field);
	}

	// Reset validation
	function reset() {
		data = initialData;
		validationResults = {} as Record<keyof T, ValidationResult>;
		isDirty = false;
	}

	// Check if form is valid
	const isValid = $derived(isFormValid(validationResults));

	return {
		data: $derived(data),
		validationResults: $derived(validationResults),
		isValid,
		isDirty: $derived(isDirty),
		validate,
		validateField,
		updateField,
		reset
	};
}
