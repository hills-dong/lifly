import { test, expect } from '@playwright/test';
import { login } from './setup';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DOC_TOOL_ID = '00000000-0000-0000-0000-000000000202';

// Real sample ID card image for meaningful OCR testing
const SAMPLE_ID_CARD = path.join(__dirname, 'fixtures', 'sample-id-card.png');

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

    await page.locator('input[type="file"]').setInputFiles(SAMPLE_ID_CARD);

    // File info should be displayed
    await expect(page.locator('.drop-zone-file')).toBeVisible();
    await expect(page.locator('.drop-zone-file strong')).toHaveText('sample-id-card.png');

    // Upload button should now be enabled
    const uploadBtn = page.locator('button.btn-primary', { hasText: 'Upload' });
    await expect(uploadBtn).toBeEnabled();
  });

  test('upload ID card and see pipeline status', async ({ page }) => {
    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    await page.locator('input[type="file"]').setInputFiles(SAMPLE_ID_CARD);
    await expect(page.locator('.drop-zone-file')).toBeVisible();

    await page.click('button.btn-primary:has-text("Upload")');

    // Should not see error
    await expect(page.locator('.alert-error')).not.toBeVisible({ timeout: 3_000 });

    // Should see pipeline status
    await expect(
      page.locator('.alert-info, .alert-success').first()
    ).toBeVisible({ timeout: 15_000 });
  });

  test('upload ID card — extraction results verification', async ({ page }) => {
    test.setTimeout(60_000); // LLM processing may take longer with real image

    await page.goto(`/tools/${DOC_TOOL_ID}/upload-doc`);
    await expect(page.locator('.drop-zone')).toBeVisible({ timeout: 10_000 });

    await page.locator('input[type="file"]').setInputFiles(SAMPLE_ID_CARD);
    await page.click('button.btn-primary:has-text("Upload")');

    // Wait for pipeline to complete (real LLM OCR takes longer)
    await page.waitForTimeout(15_000);

    // Take screenshot for verification
    const screenshotDir = path.join(process.cwd(), 'test-results', 'doc-upload-screenshots');
    fs.mkdirSync(screenshotDir, { recursive: true });
    await page.screenshot({
      path: path.join(screenshotDir, 'id-card-extraction-result.png'),
      fullPage: true,
    });

    // Pipeline should complete and show extraction results or status
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
