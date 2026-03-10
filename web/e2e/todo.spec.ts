import { test, expect } from '@playwright/test';
import { login } from './setup';

const TODO_TOOL_ID = '00000000-0000-0000-0000-000000000201';

test.describe('Todo Tool', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to Todo tool page', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('h1')).toContainText('Todo', { timeout: 10_000 });
    // Should have a "New Todo" button
    await expect(page.locator('a.btn', { hasText: 'New Todo' })).toBeVisible();
  });

  test('open new todo form', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('h1')).toHaveText('New Todo', { timeout: 10_000 });
    await expect(page.locator('#content')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('submit a todo and see pipeline status', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('#content')).toBeVisible({ timeout: 10_000 });

    await page.fill('#content', 'E2E test todo item');
    await page.click('button[type="submit"]');

    // After submit, should see pipeline status feedback (submitted or further)
    // The alert-info or alert-success should appear
    await expect(
      page.locator('.alert-info, .alert-success, .alert').first()
    ).toBeVisible({ timeout: 15_000 });
  });
});
