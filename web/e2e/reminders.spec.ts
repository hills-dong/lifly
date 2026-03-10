import { test, expect } from '@playwright/test';
import { login } from './setup';

test.describe('Reminders', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('navigate to reminders page', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('h1')).toHaveText('Reminders', { timeout: 10_000 });
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible();
  });

  test('toggle new reminder form visibility', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    // Form should not be visible initially
    await expect(page.locator('#r-title')).not.toBeVisible();

    // Click to open form
    await page.click('button:has-text("New Reminder")');
    await expect(page.locator('#r-title')).toBeVisible();

    // Button text changes to "Cancel"
    await expect(page.locator('button', { hasText: 'Cancel' }).first()).toBeVisible();

    // Click again to close form
    await page.click('button:has-text("Cancel")');
    await expect(page.locator('#r-title')).not.toBeVisible();
  });

  test('create a new reminder', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    await page.click('button:has-text("New Reminder")');
    await expect(page.locator('#r-title')).toBeVisible({ timeout: 5_000 });

    const reminderTitle = `E2E Reminder ${Date.now()}`;
    await page.fill('#r-title', reminderTitle);
    await page.fill('#r-desc', 'Created by Playwright');

    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    await page.fill('#r-due', tomorrow.toISOString().slice(0, 16));

    await page.click('form button[type="submit"]');

    // Form should close and reminder should appear in list
    await expect(page.locator('#r-title')).not.toBeVisible({ timeout: 5_000 });
    await expect(page.locator('.reminder-list')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator(`text=${reminderTitle}`).first()).toBeVisible();
  });

  test('reminder shows title, description, and due date', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    // Create a reminder with known content
    await page.click('button:has-text("New Reminder")');
    const title = `Detail Check ${Date.now()}`;
    await page.fill('#r-title', title);
    await page.fill('#r-desc', 'Test description content');
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    await page.fill('#r-due', tomorrow.toISOString().slice(0, 16));
    await page.click('form button[type="submit"]');

    await expect(page.locator('.reminder-list')).toBeVisible({ timeout: 10_000 });

    // Find the reminder item
    const item = page.locator('.reminder-item', { hasText: title });
    await expect(item).toBeVisible();
    await expect(item.locator('strong')).toHaveText(title);
    await expect(item).toContainText('Test description content');
    await expect(item.locator('.reminder-due')).toBeVisible();
    await expect(item.locator('.badge')).toBeVisible();
  });

  test('dismiss a reminder', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    // Create a reminder to dismiss
    await page.click('button:has-text("New Reminder")');
    const title = `Dismiss Me ${Date.now()}`;
    await page.fill('#r-title', title);
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    await page.fill('#r-due', tomorrow.toISOString().slice(0, 16));
    await page.click('form button[type="submit"]');

    await expect(page.locator('.reminder-list')).toBeVisible({ timeout: 10_000 });

    // Find the reminder and click Dismiss
    const item = page.locator('.reminder-item', { hasText: title });
    await expect(item).toBeVisible();
    await item.locator('button:has-text("Dismiss")').click();

    // After dismiss, the dismiss button should be gone for this item
    // (page refetches, dismissed items may still show but without dismiss button)
    await page.waitForTimeout(1000);
    // The item should still exist but with dismissed status
    const dismissedItem = page.locator('.reminder-item', { hasText: title });
    if (await dismissedItem.isVisible()) {
      await expect(dismissedItem.locator('.badge')).toContainText('dismissed');
      await expect(dismissedItem.locator('button:has-text("Dismiss")')).not.toBeVisible();
    }
  });

  test('delete a reminder with confirmation', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    // Create a reminder to delete
    await page.click('button:has-text("New Reminder")');
    const title = `Delete Me ${Date.now()}`;
    await page.fill('#r-title', title);
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    await page.fill('#r-due', tomorrow.toISOString().slice(0, 16));
    await page.click('form button[type="submit"]');

    await expect(page.locator('.reminder-list')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.reminder-item', { hasText: title })).toBeVisible();

    // Accept confirmation dialog and delete
    page.on('dialog', (dialog) => dialog.accept());
    await page.locator('.reminder-item', { hasText: title }).locator('button:has-text("Delete")').click();

    // The deleted reminder should disappear
    await expect(page.locator('.reminder-item', { hasText: title })).not.toBeVisible({ timeout: 5_000 });
  });

  test('filter reminders by status', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('h1')).toHaveText('Reminders', { timeout: 10_000 });

    // The filter dropdown should exist
    const filter = page.locator('.filter-bar select');
    await expect(filter).toBeVisible();

    // Switch to "Pending" filter
    await filter.selectOption('pending');
    await page.waitForTimeout(500);

    // Switch to "Dismissed" filter
    await filter.selectOption('dismissed');
    await page.waitForTimeout(500);

    // Switch back to "All"
    await filter.selectOption('');
    await page.waitForTimeout(500);

    // Should not crash — page renders correctly
    await expect(page.locator('h1')).toHaveText('Reminders');
  });
});
