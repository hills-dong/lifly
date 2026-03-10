import { test, expect } from '@playwright/test';
import { login, apiLogin, createTodoViaApi } from './setup';

const TODO_TOOL_ID = '00000000-0000-0000-0000-000000000201';

test.describe('Todo Tool', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to Todo tool page', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('h1')).toContainText('Todo', { timeout: 10_000 });
    await expect(page.locator('a.btn', { hasText: 'New Todo' })).toBeVisible();
  });

  test('open new todo form', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('h1')).toHaveText('New Todo', { timeout: 10_000 });
    await expect(page.locator('#content')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('empty content cannot be submitted', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('#content')).toBeVisible({ timeout: 10_000 });

    // Leave input empty and click submit
    await page.click('button[type="submit"]');

    // Should stay on the same page (HTML5 required validation prevents submit)
    await expect(page).toHaveURL(/new-todo/);
    // No alert should appear (form was never submitted)
    await expect(page.locator('.alert-error')).not.toBeVisible();
    await expect(page.locator('.alert-info')).not.toBeVisible();
  });

  test('submit a todo and see pipeline status', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('#content')).toBeVisible({ timeout: 10_000 });

    await page.fill('#content', 'E2E test todo item');
    await page.click('button[type="submit"]');

    // Must see success or in-progress status, NOT an error
    await expect(page.locator('.alert-error')).not.toBeVisible({ timeout: 3_000 });
    await expect(
      page.locator('.alert-info, .alert-success').first()
    ).toBeVisible({ timeout: 15_000 });
  });

  test('submit todo and redirect to list with new item', async ({ page }) => {
    await page.goto(`/tools/${TODO_TOOL_ID}/new-todo`);
    await expect(page.locator('#content')).toBeVisible({ timeout: 10_000 });

    const todoText = `E2E redirect test ${Date.now()}`;
    await page.fill('#content', todoText);
    await page.click('button[type="submit"]');

    // Wait for pipeline to complete and auto-redirect to tool page
    await expect(page).toHaveURL(`/tools/${TODO_TOOL_ID}`, { timeout: 20_000 });

    // The new todo should appear in the list
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.todo-item', { hasText: todoText })).toBeVisible({ timeout: 10_000 });
  });

  test('toggle todo done status', async ({ page, request }) => {
    // Create a todo via API first
    const baseURL = page.url().startsWith('http') ? new URL(page.url()).origin : 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E toggle test ${Date.now()}`);

    // Navigate to tool page
    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });

    // Find the first unchecked todo and toggle it
    const firstCheckbox = page.locator('.todo-item input[type="checkbox"]').first();
    const wasChecked = await firstCheckbox.isChecked();
    await firstCheckbox.click();

    // Verify state changed
    if (wasChecked) {
      await expect(firstCheckbox).not.toBeChecked();
    } else {
      await expect(firstCheckbox).toBeChecked();
    }
  });

  test('filter todos by status', async ({ page, request }) => {
    // Ensure at least one todo exists
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E filter test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });

    // Filter by "completed" — should not crash, page renders
    await page.selectOption('select', 'completed');
    await page.waitForTimeout(1000);
    await expect(page.locator('h1')).toContainText('Todo');

    // Filter by "active" — should show items
    await page.selectOption('select', 'active');
    await page.waitForTimeout(1000);
    await expect(page.locator('.todo-list')).toBeVisible();

    // Filter back to "all"
    await page.selectOption('select', '');
    await page.waitForTimeout(1000);
    await expect(page.locator('.todo-list')).toBeVisible();
  });

  test('click View navigates to data object detail', async ({ page, request }) => {
    // Ensure at least one todo exists
    const baseURL = page.url().startsWith('http') ? new URL(page.url()).origin : 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E detail test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });

    // Click the first "View" link
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();

    // Should navigate to a data-objects detail page
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });
  });
});
