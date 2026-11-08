import { expect, test } from '@playwright/test';

test('home page loads and renders the overview heading', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Overview', exact: true })).toBeVisible();
});
