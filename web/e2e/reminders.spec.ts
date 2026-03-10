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

  test('create a new reminder', async ({ page }) => {
    await page.goto('/reminders');
    await expect(page.locator('button', { hasText: 'New Reminder' })).toBeVisible({ timeout: 10_000 });

    // Open the form
    await page.click('button:has-text("New Reminder")');
    await expect(page.locator('#r-title')).toBeVisible({ timeout: 5_000 });

    // Fill in reminder details
    await page.fill('#r-title', 'E2E Test Reminder');
    await page.fill('#r-desc', 'Created by Playwright E2E test');

    // Set due date to tomorrow
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    const dateStr = tomorrow.toISOString().slice(0, 16); // yyyy-MM-ddTHH:mm
    await page.fill('#r-due', dateStr);

    // Submit
    await page.click('form button[type="submit"]');

    // Should see the reminder in the list
    await expect(page.locator('.reminder-list')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('text=E2E Test Reminder').first()).toBeVisible();
  });
});
