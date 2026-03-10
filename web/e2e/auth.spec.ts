import { test, expect } from '@playwright/test';
import { login } from './setup';

test.describe('Authentication', () => {
  test('unauthenticated users are redirected to login', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL(/login/);
  });

  test('login with valid credentials shows tools', async ({ page }) => {
    await login(page);
    await expect(page).toHaveURL('/');
    await expect(page.locator('h1')).toHaveText('Tools');
    // Should see at least one tool card
    await expect(page.locator('.card-grid .card-link').first()).toBeVisible({ timeout: 10_000 });
  });

  test('login with invalid credentials stays on login page', async ({ page }) => {
    await page.goto('/login');
    await page.fill('#username', 'admin');
    await page.fill('#password', 'wrongpassword');
    await page.click('button[type="submit"]');
    // Should stay on login page — wait a bit to confirm no redirect
    await page.waitForTimeout(2_000);
    await expect(page).toHaveURL(/login/);
    // The page should still show the login form (error may or may not render depending on API response timing)
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });
});
