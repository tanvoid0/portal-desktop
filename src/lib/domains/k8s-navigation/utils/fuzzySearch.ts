// Fuzzy search algorithm for commands and resources

export interface SearchResult<T> {
	item: T;
	score: number;
	matches: number[];
}

export function fuzzySearch<T>(
	query: string,
	items: T[],
	getText: (item: T) => string,
	getKeywords?: (item: T) => string[]
): SearchResult<T>[] {
	if (!query.trim()) {
		return items.map((item, index) => ({
			item,
			score: 1,
			matches: []
		}));
	}
	
	const queryLower = query.toLowerCase();
	const results: SearchResult<T>[] = [];
	
	for (const item of items) {
		const text = getText(item).toLowerCase();
		const keywords = getKeywords ? getKeywords(item).join(' ').toLowerCase() : '';
		const searchable = `${text} ${keywords}`;
		
		let score = 0;
		const matches: number[] = [];
		let queryIndex = 0;
		let lastMatchIndex = -1;
		
		// Check for exact match
		if (text.includes(queryLower)) {
			score += 100;
		}
		
		// Check for starts with
		if (text.startsWith(queryLower)) {
			score += 50;
		}
		
		// Fuzzy match - find all characters in order
		for (let i = 0; i < searchable.length && queryIndex < queryLower.length; i++) {
			if (searchable[i] === queryLower[queryIndex]) {
				matches.push(i);
				queryIndex++;
				lastMatchIndex = i;
			}
		}
		
		// If all characters matched, calculate score
		if (queryIndex === queryLower.length) {
			// Bonus for consecutive matches
			let consecutiveBonus = 0;
			for (let i = 1; i < matches.length; i++) {
				if (matches[i] === matches[i - 1] + 1) {
					consecutiveBonus += 5;
				}
			}
			
			// Bonus for matches at word boundaries
			let wordBoundaryBonus = 0;
			for (const match of matches) {
				if (match === 0 || searchable[match - 1] === ' ') {
					wordBoundaryBonus += 10;
				}
			}
			
			// Penalty for distance between matches
			const distance = lastMatchIndex - matches[0];
			const distancePenalty = Math.max(0, distance - queryLower.length) * 0.5;
			
			score += consecutiveBonus + wordBoundaryBonus - distancePenalty;
			
			results.push({ item, score, matches });
		}
	}
	
	// Sort by score descending
	results.sort((a, b) => b.score - a.score);
	
	return results;
}

export function highlightMatches(text: string, matches: number[]): string {
	if (matches.length === 0) return text;
	
	const parts: string[] = [];
	let lastIndex = 0;
	
	for (const matchIndex of matches) {
		if (matchIndex > lastIndex) {
			parts.push(text.slice(lastIndex, matchIndex));
		}
		parts.push(`<mark>${text[matchIndex]}</mark>`);
		lastIndex = matchIndex + 1;
	}
	
	if (lastIndex < text.length) {
		parts.push(text.slice(lastIndex));
	}
	
	return parts.join('');
}

