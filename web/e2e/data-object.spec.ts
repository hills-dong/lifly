import { test, expect } from '@playwright/test';
import { login, apiLogin, createTodoViaApi } from './setup';

const TODO_TOOL_ID = '00000000-0000-0000-0000-000000000201';

test.describe('Data Object Detail', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('view data object detail from todo list', async ({ page, request }) => {
    // Create a todo via API
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E detail view ${Date.now()}`);

    // Navigate to todo list and click View
    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();

    // Should be on data object detail page
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Should show status and attributes
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('.detail-grid')).toBeVisible();
    await expect(page.locator('.attributes-view')).toBeVisible();
  });

  test('display attributes as key-value pairs', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E attrs test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Attributes section should have at least one key-value row
    await expect(page.locator('.attributes-view .detail-row').first()).toBeVisible();
    await expect(page.locator('.attributes-view .detail-label').first()).toBeVisible();
  });

  test('edit attributes in JSON editor', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E edit test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Click Edit button
    await page.click('button:has-text("Edit")');
    await expect(page.locator('h1')).toHaveText('Edit Item');
    await expect(page.locator('#attrs')).toBeVisible();

    // The textarea should contain valid JSON
    const attrsValue = await page.locator('#attrs').inputValue();
    expect(() => JSON.parse(attrsValue)).not.toThrow();

    // Cancel should return to view mode
    await page.click('button:has-text("Cancel")');
    await expect(page.locator('.attributes-view')).toBeVisible();
  });

  test('save edited attributes', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    const uniqueTag = `edited-${Date.now()}`;
    await createTodoViaApi(request, baseURL, token, `E2E save test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Edit attributes
    await page.click('button:has-text("Edit")');
    await expect(page.locator('#attrs')).toBeVisible();

    // Replace with new JSON including a unique tag
    await page.locator('#attrs').fill(JSON.stringify({ content: uniqueTag }));
    await page.click('button:has-text("Save")');

    // Should return to view mode and show updated attribute
    await expect(page.locator('.attributes-view')).toBeVisible({ timeout: 5_000 });
    await expect(page.locator('.attributes-view')).toContainText(uniqueTag);
  });

  test('back to tool link works', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E back test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Click back link
    await page.click('.back-link');
    await expect(page).toHaveURL(`/tools/${TODO_TOOL_ID}`, { timeout: 5_000 });
  });

  test('delete data object with confirmation', async ({ page, request }) => {
    const baseURL = 'http://localhost:9527';
    const token = await apiLogin(request, baseURL);
    await createTodoViaApi(request, baseURL, token, `E2E delete test ${Date.now()}`);

    await page.goto(`/tools/${TODO_TOOL_ID}`);
    await expect(page.locator('.todo-list')).toBeVisible({ timeout: 10_000 });

    const countBefore = await page.locator('.todo-item').count();

    // Go to detail of first item
    await page.locator('.todo-item .btn', { hasText: 'View' }).first().click();
    await expect(page).toHaveURL(/\/data-objects\//, { timeout: 5_000 });

    // Accept the confirmation dialog and click delete
    page.on('dialog', (dialog) => dialog.accept());
    await page.click('button:has-text("Delete")');

    // Should navigate back
    await expect(page).not.toHaveURL(/\/data-objects\//, { timeout: 5_000 });
  });
});
