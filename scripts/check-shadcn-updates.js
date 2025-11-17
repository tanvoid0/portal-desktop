#!/usr/bin/env node
/**
 * Script to check which shadcn components might need updates
 * Run this before doing a full reinit to see what would change
 */

import { readdir, readFile } from 'fs/promises';
import { join } from 'path';
import { existsSync } from 'fs';

const UI_DIR = 'src/lib/components/ui';
const COMPONENTS_TO_CHECK = [
	'button',
	'card',
	'input',
	'select',
	'dialog',
	'dropdown-menu',
	'tooltip',
	'sidebar',
	'badge',
	'alert',
	'form'
];

async function checkComponent(componentName) {
	const componentDir = join(UI_DIR, componentName);
	if (!existsSync(componentDir)) {
		return { name: componentName, exists: false };
	}

	try {
		const files = await readdir(componentDir);
		const svelteFiles = files.filter(f => f.endsWith('.svelte'));
		
		// Check for custom modifications
		let hasCustomCode = false;
		let usesRunes = false;
		
		for (const file of svelteFiles) {
			const content = await readFile(join(componentDir, file), 'utf-8');
			if (content.includes('$derived') || content.includes('$state') || content.includes('$props')) {
				usesRunes = true;
			}
			// Check for custom comments or modifications
			if (content.includes('CUSTOM') || content.includes('MODIFIED') || content.includes('TODO')) {
				hasCustomCode = true;
			}
		}

		return {
			name: componentName,
			exists: true,
			files: svelteFiles.length,
			usesRunes,
			hasCustomCode
		};
	} catch (error) {
		return { name: componentName, error: error.message };
	}
}

async function main() {
	console.log('🔍 Checking shadcn components for potential update issues...\n');
	
	const results = await Promise.all(
		COMPONENTS_TO_CHECK.map(comp => checkComponent(comp))
	);

	console.log('Component Status:\n');
	results.forEach(result => {
		if (!result.exists) {
			console.log(`  ❌ ${result.name}: Not found`);
		} else if (result.error) {
			console.log(`  ⚠️  ${result.name}: Error - ${result.error}`);
		} else {
			const status = [];
			if (result.usesRunes) status.push('✅ Svelte 5');
			if (result.hasCustomCode) status.push('⚠️  Custom code');
			console.log(`  ${status.join(' | ') || '✅'} ${result.name} (${result.files} files)`);
		}
	});

	console.log('\n💡 Recommendation:');
	console.log('  - Only update components that are actually broken');
	console.log('  - Test each component after update');
	console.log('  - Keep a backup of custom modifications');
	console.log('\n📝 To update a single component:');
	console.log('  npx shadcn-svelte@latest add <component-name> --overwrite');
}

main().catch(console.error);

