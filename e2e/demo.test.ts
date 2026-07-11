import { expect, test } from '@playwright/test';

test('home page loads and renders the dashboard heading', async ({ page }) => {
	await page.goto('/');
	await expect(
		page.getByRole('heading', { name: /^Good (morning|afternoon|evening)!$/ }),
	).toBeVisible();
});
