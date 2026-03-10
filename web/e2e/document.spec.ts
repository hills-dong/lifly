import { test, expect } from '@playwright/test';
import { login } from './setup';
import path from 'path';
import fs from 'fs';
import os from 'os';

const DOC_TOOL_ID = '00000000-0000-0000-0000-000000000202';

// Create a minimal test PNG file for upload tests
function createTestPng(): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'e2e-'));
  const filePath = path.join(dir, 'test-doc.png');
  const pngData = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==',
    'base64'
  );
  fs.writeFileSync(filePath, pngData);
  return filePath;
}

test.describe('Document Upload', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to document upload page', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('h1')).toHaveText('Upload Document', { timeout: 10_000 });
  });

  test('drop zone is visible with prompt text', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.drop-zone-prompt')).toBeVisible();
    await expect(page.locator('.drop-zone-prompt')).toContainText('Drag and drop');
  });

  test('upload button is disabled without file', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    const uploadBtn = page.locator('button.btn-primary', { hasText: 'Upload' });
    await expect(uploadBtn).toBeDisabled();
  });

  test('select file via input shows file info', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    const testPng = createTestPng();
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles(testPng);

    // File info should be displayed
    await expect(page.locator('.drop-zone-file')).toBeVisible();
    await expect(page.locator('.drop-zone-file strong')).toHaveText('test-doc.png');

    // Upload button should now be enabled
    const uploadBtn = page.locator('button.btn-primary', { hasText: 'Upload' });
    await expect(uploadBtn).toBeEnabled();
  });

  test('upload file and see pipeline status', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    const testPng = createTestPng();
    await page.locator('input[type="file"]').setInputFiles(testPng);
    await expect(page.locator('.drop-zone-file')).toBeVisible();

    // Click upload
    await page.click('button.btn-primary:has-text("Upload")');

    // Should not see error
    await expect(page.locator('.alert-error')).not.toBeVisible({ timeout: 3_000 });

    // Should see pipeline status (submitted, processing, or completed)
    await expect(
      page.locator('.alert-info, .alert-success').first()
    ).toBeVisible({ timeout: 15_000 });
  });

  test('upload completes — capture result for verification', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    const testPng = createTestPng();
    await page.locator('input[type="file"]').setInputFiles(testPng);
    await page.click('button.btn-primary:has-text("Upload")');

    // Wait for pipeline to finish
    await page.waitForTimeout(10_000);

    // Take screenshot for manual verification of extraction results
    const screenshotDir = path.join(process.cwd(), 'test-results', 'doc-upload-screenshots');
    fs.mkdirSync(screenshotDir, { recursive: true });
    await page.screenshot({
      path: path.join(screenshotDir, 'doc-upload-result.png'),
      fullPage: true,
    });

    // At minimum, no crash — either extraction results page or pipeline status visible
    const hasResults = await page.locator('h1:has-text("Extraction Results")').isVisible();
    const hasStatus = await page.locator('.alert-info, .alert-success').first().isVisible();
    expect(hasResults || hasStatus).toBeTruthy();
  });

  test('navigate to doc tool page shows Upload Document button', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}`);
    await expect(page.locator('h1')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('a.btn', { hasText: 'Upload Document' })).toBeVisible();
  });
});
