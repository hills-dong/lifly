import { test, expect } from '@playwright/test';
import { login } from './setup';

const DOC_TOOL_ID = '00000000-0000-0000-0000-000000000202';

test.describe('Document Upload', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to document upload page', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('h1')).toHaveText('Upload Document', { timeout: 10_000 });
  });

  test('drop zone is visible', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.drop-zone-prompt')).toBeVisible();
  });
});
