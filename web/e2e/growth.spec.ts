import { test, expect } from '@playwright/test';
import { login } from './setup';

// The 成长记录 tool renders through the embedded tool view (the same component the
// native iOS shell hosts), reachable in a browser at /embed/tools/:id.
const GROWTH_TOOL_ID = '00000000-0000-0000-0000-000000000203';

test.describe('Growth Record Tool', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('shows the latest measurement and all 41 records', async ({ page }) => {
    await page.goto(`/embed/tools/${GROWTH_TOOL_ID}`);

    const summary = page.locator('.growth-summary');
    await expect(summary).toBeVisible({ timeout: 10_000 });
    await expect(summary).toContainText('115');
    await expect(summary).toContainText('17');
    await expect(summary).toContainText('5岁8月6天');

    await expect(page.locator('.growth-row')).toHaveCount(41);
    // Newest first.
    await expect(page.locator('.growth-row').first()).toContainText('2025-11-19');
    await expect(page.locator('.growth-row').last()).toContainText('2020-04-03');
  });

  test('classifies a low weight measurement against WHO bands', async ({ page }) => {
    await page.goto(`/embed/tools/${GROWTH_TOOL_ID}`);
    await expect(page.locator('.growth-summary')).toBeVisible({ timeout: 10_000 });

    // 2024-08-10: 107cm (正常) / 14.5kg (偏低) per the WHO percentile zones.
    const row = page.locator('.growth-row', { hasText: '2024-08-10' });
    const metrics = row.locator('.growth-metric');
    await expect(metrics.nth(0)).toContainText('正常');
    await expect(metrics.nth(1)).toContainText('偏低');
  });

  test('renders the height curve with WHO percentile bands and the child line', async ({ page }) => {
    await page.goto(`/embed/tools/${GROWTH_TOOL_ID}`);
    await expect(page.locator('.growth-summary')).toBeVisible({ timeout: 10_000 });

    await page.getByRole('button', { name: '身高曲线' }).click();
    const chart = page.locator('svg.growth-chart');
    await expect(chart).toBeVisible();
    // P3/P15/P50/P85/P97 reference lines.
    await expect(chart.locator('.growth-pline')).toHaveCount(5);
    await expect(chart.locator('.growth-childline')).toBeVisible();
    expect(await chart.locator('.growth-dot').count()).toBeGreaterThan(20);
  });

  test('switches to the weight curve and toggles sex', async ({ page }) => {
    await page.goto(`/embed/tools/${GROWTH_TOOL_ID}`);
    await expect(page.locator('.growth-summary')).toBeVisible({ timeout: 10_000 });

    await page.getByRole('button', { name: '体重曲线' }).click();
    await expect(page.locator('svg.growth-chart')).toBeVisible();

    await page.getByRole('button', { name: '女孩' }).click();
    await expect(page.locator('svg.growth-chart')).toBeVisible();
    await expect(page.locator('.growth-childline')).toBeVisible();
  });
});
