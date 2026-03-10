import { type Page, expect } from '@playwright/test';

export async function login(page: Page) {
  await page.goto('/login');
  await page.fill('#username', 'admin');
  await page.fill('#password', 'admin123');
  await page.click('button[type="submit"]');
  await expect(page).not.toHaveURL(/login/, { timeout: 10_000 });
}
