import { test, expect } from '@playwright/test';
import { login } from './setup';

test.describe('Search', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to search page and see search input', async ({ page }) => {
    await page.goto('/search');
    await expect(page.locator('h1')).toHaveText('Search', { timeout: 10_000 });
    await expect(page.locator('.search-input')).toBeVisible();
  });

  test('search for nonexistent term shows no results', async ({ page }) => {
    await page.goto('/search');
    await expect(page.locator('.search-input')).toBeVisible({ timeout: 10_000 });

    await page.fill('.search-input', 'zzz_nonexistent_item_xyz');
    await page.click('button[type="submit"]');

    await expect(page.locator('.empty-state')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.empty-state')).toContainText('No results');
  });
});
