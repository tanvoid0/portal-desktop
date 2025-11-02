/**
 * Format bytes to human readable format
 * @param bytes - Number of bytes
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted string (e.g., "1.5 GB", "500 MB")
 */
export function formatFileSize(bytes: number, decimals: number = 2): string {
	if (bytes === 0) return '0 Bytes';
	
	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
	
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	
	return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

/**
 * Parse size string from Ollama API response
 * @param sizeString - Size string or number from API (e.g., "3.8GB", "500MB", 1234567890)
 * @returns Formatted human readable size
 */
export function parseAndFormatSize(sizeString: string | number): string {
	// If it's already a number, use it directly
	if (typeof sizeString === 'number') {
		return formatFileSize(sizeString);
	}
	
	// Remove any non-numeric characters except decimal point
	const numericPart = sizeString.replace(/[^\d.]/g, '');
	const bytes = parseFloat(numericPart);
	
	// If the original string contains GB, MB, KB, multiply accordingly
	if (sizeString.toLowerCase().includes('gb')) {
		return formatFileSize(bytes * 1024 * 1024 * 1024);
	} else if (sizeString.toLowerCase().includes('mb')) {
		return formatFileSize(bytes * 1024 * 1024);
	} else if (sizeString.toLowerCase().includes('kb')) {
		return formatFileSize(bytes * 1024);
	} else {
		// Assume it's already in bytes
		return formatFileSize(bytes);
	}
}
