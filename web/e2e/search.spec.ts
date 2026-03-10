import { test, expect } from '@playwright/test';
import { login, apiLogin, createTodoViaApi } from './setup';

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

  test('empty query does not trigger search', async ({ page }) => {
    await page.goto('/search');
    await expect(page.locator('.search-input')).toBeVisible({ timeout: 10_000 });

    // Leave empty and submit
    await page.click('button[type="submit"]');

    // Should not show "No results" (search was not triggered)
    await expect(page.locator('.empty-state')).not.toBeVisible({ timeout: 2_000 });
    await expect(page.locator('.table')).not.toBeVisible();
  });

  test('search with valid term returns results', async ({ page, request }) => {
    // Create a todo with a unique searchable term
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    const searchTerm = `searchable-${Date.now()}`;
    await createTodoViaApi(request, baseURL, token, searchTerm);

    await page.goto('/search');
    await expect(page.locator('.search-input')).toBeVisible({ timeout: 10_000 });

    await page.fill('.search-input', searchTerm);
    await page.click('button[type="submit"]');

    // Should show results table
    await expect(page.locator('.table')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.table tbody tr').first()).toBeVisible();
  });

  test('click View in search results navigates to detail', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    const searchTerm = `navigate-${Date.now()}`;
    await createTodoViaApi(request, baseURL, token, searchTerm);

    await page.goto('/search');
    await page.fill('.search-input', searchTerm);
    await page.click('button[type="submit"]');

    await expect(page.locator('.table')).toBeVisible({ timeout: 10_000 });

    // Click View on first result
    await page.locator('.table tbody tr .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });
  });
});
